export const meta = {
  name: "implement-task",
  description: "タスクを実装する標準ワークフロー。探索→計画→実装→検証の4フェーズで進める（asmr-dl / Rust）。",
  whenToUse: "新しい機能追加やバグ修正タスクを実装する時に使用。",
  phases: [
    { title: "Explore", detail: "コードベースの探索と現状把握" },
    { title: "Plan", detail: "実装計画の立案と設計判断（モジュール境界・検証方法を含む）" },
    { title: "Implement", detail: "実装とテスト作成（純粋関数の分離）" },
    { title: "Verify", detail: "検証とレビュー（cargo が使える環境 / Sandbox では §6.2 の代替検証）" }
  ]
};

const A = typeof args === "string" ? JSON.parse(args) : (args || {});
const task = A.task || A.taskId || "";

if (!task) {
  return {
    status: "error",
    message: "タスクIDまたはタスク内容を指定してください。例: { \"task\": \"PROFILE-1: bilibili の並列数を増加\" }"
  };
}

const EXPLORE_SCHEMA = {
  type: "object",
  required: ["files", "summary"],
  properties: {
    files: {
      type: "array",
      items: { type: "string" },
      description: "関連ファイルのリスト（src/*.rs, config.default.toml, docs/**, .agent/**）"
    },
    summary: {
      type: "string",
      description: "現状の要約（該当モジュール・プロファイル・既存テスト）"
    }
  }
};

const PLAN_SCHEMA = {
  type: "object",
  required: ["scope", "prohibitions", "completionCriteria", "approach", "verification"],
  properties: {
    scope: {
      type: "array",
      items: { type: "string" },
      description: "変更範囲（モジュール / 設定 / docs）"
    },
    prohibitions: {
      type: "array",
      items: { type: "string" },
      description: "禁止事項（推測で埋めない・generic末尾・既定値変更なし・asmr-dl.zip/target不変）"
    },
    completionCriteria: {
      type: "array",
      items: { type: "string" },
      description: "完了条件（第三者がYes/No判定できる形）"
    },
    approach: {
      type: "string",
      description: "実装アプローチ（ロジックの純粋関数分離を含む）"
    },
    verification: {
      type: "string",
      description: "検証方法（cargo fmt/clippy/test/build か、§6.2 の代替検証 + 実機検証の分離）"
    }
  }
};

const VERIFY_SCHEMA = {
  type: "object",
  required: ["results", "passed", "unverified"],
  properties: {
    results: {
      type: "array",
      items: {
        type: "object",
        required: ["command", "ok"],
        properties: {
          command: { type: "string", description: "実行コマンド" },
          ok: { type: "boolean", description: "PASS/FAIL" }
        }
      },
      description: "検証結果のリスト（cargo fmt/clippy/test/build か、代替検証）"
    },
    passed: { type: "boolean", description: "全検証がPASSしたか" },
    unverified: {
      type: "array",
      items: { type: "string" },
      description: "実行できなかった検証（Sandbox §6.2 / 実機検証待ちの device-testing D番号）"
    }
  }
};

export default async function run(agent, parallel, phase) {
  // Phase 1: Explore
  const explore = await agent(
    {
      role: "explore",
      task: `タスク「${task}」に関するコードベースを探索してください。src/*.rs, config.default.toml, docs/**, .agent/** を対象に、関連ファイルと現状を要約してください。`,
      schema: EXPLORE_SCHEMA
    }
  );

  // Phase 2: Plan
  const plan = await agent(
    {
      role: "plan",
      task: `タスク「${task}」の実装計画を立案してください。探索結果: ${JSON.stringify(explore)}。モジュール境界（import-boundaries）・Sandbox 制約（AGENTS.md §6.2）・検証方法（cargo か代替検証+実機検証）を必ず含めてください。`,
      schema: PLAN_SCHEMA
    }
  );

  // Phase 3: Implement
  const implement = await agent(
    {
      role: "implementer",
      task: `計画に従って実装してください。計画: ${JSON.stringify(plan)}。純粋関数は #[cfg(test)] mod tests でテストを追加してください（testing スキル）。config.default.toml を変える場合は docs/arch/profiles.md と docs/examples/ を同期してください。`
    }
  );

  // Phase 4: Verify
  const verify = await agent(
    {
      role: "verifier",
      task: `実装を検証してください。計画の検証方法: ${plan.verification}。cargo が使える環境では fmt/clippy/test/build を、Sandbox（cargo 無し）では §6.2 の代替検証（tomllib / sh -n / jq / リンクチェック）を実施してください。実行できなかった検証（実機検証待ちの device-testing D番号を含む）を unverified に列挙してください。`,
      schema: VERIFY_SCHEMA
    }
  );

  return {
    status: verify.passed ? "success" : "failed",
    explore,
    plan,
    implement,
    verify,
    next: verify.unverified.length > 0
      ? `以下の検証が未実行です（残課題として docs/task-list.md に記録してください）: ${verify.unverified.join(", ")}`
      : "全検証完了。commit/push を行ってください。"
  };
}
