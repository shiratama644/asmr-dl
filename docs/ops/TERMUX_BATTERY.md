# Ops: Termux での長時間ダウンロードと電力管理

> **対象**: 充電しながら数時間の ASMR ダウンロードを安定させる電力管理（**第1人称実行手順**）。
> **事前条件**: `INSTALL_TERMUX.md` を完了済み（Termux:API パッケージ + アプリ があることを確認）。

## 0. 何が起きているか（前提）

- asmr-dl は起動時に `termux-wake-lock` を呼び、ダウンロード完了・全キャンセルで `termux-wake-unlock`（`src/main.rs` / `wake-lock.rs` 相当）。
- **Termux:API アプリが無いと `termux-wake-lock` は失敗**。asmr-dl はそれを `warn!` として処理し、続行する（本体は動く）。
- Android 本体が Termux プロセスを**最適化（kill）すると、wake-lock されていてもダウンロードが途中で止まる**。
- したがって、安定させるには **① Termux:API アプリの設置 + ② Android 側のバッテリー最適化の除外** の両方が必要。

## 1. Termux:API アプリの設置

1. Play ストア（または F-Droid）で「**Termux:API**」アプリをインストールする（パッケージ `com.termux.api`）。
2. `pkg install termux-api`（未導入なら）→ 再起動は不要。
3. 確認:

```bash
termux-wake-lock && echo "wake-lock OK" && termux-wake-unlock
termux-notification -t "test" "ASM R DL 動作確認" && echo "通知 OK"
```

- `wake-lock OK` / `通知 OK` が両方出れば、アプリが動いている。
- 出ない場合:
  - `pm list packages | grep termux` に `com.termux` と `com.termux.api` があるか確認
  - タスクマネージャから Termux:API を強制終了して再実行
  - Android のセキュリティ設定で「Termux が termux:api を起動」を許可

## 2. Android 側のバッテリー最適化を除外（**重要**）

> この手順をしないと、充電中でも数十分〜数時間で Termux が kill され、ダウンロードが途中で止まる（ユーザー報告が多い）。

### Android 10 以上（Pixel 例）

1. **設定** → **アプリと通知** → **すべて表示** → **Termux** を開く
2. **バッテリー** → **バッテリーの最適化** を開く
3. 「すべてのアプリ」リストから **Termux** を探し、**「制限なし」** を選ぶ
4. **Termux:API** も同様に「制限なし」にしておく
5. （任意だが推奨）上画面の「**この端末の設定を編集**」は必須項目なし

### 一般的な Android（Xiaomi / Samsung / 国産等）

- Xiaomi: **設定** → **アプリ** → **Termux** → **バッテリーの使用を管理** → 「**バックグラウンド実行の制限なし**」+「**すべてのバックグラウンドアクセスを許可**」
- Samsung: **設定** → **バッテリーと充電** → **バックグラウンド使い過ぎ防止** に **Termux を追加**（＝除外）
- 国産: 「**省電力**」系をオフ、または **Termux を「省電力の対象外」** に

### 確認（Termux 側）

```bash
# 端末のロック状態 / 画面状態を参照（参考）
termux-battery-status
termux-wake-lock && echo "ロック中"
```

## 3. 充電接続の確認

- 充電器は**充電対応（出力電流）のもの**を使う（純正または認証済み）。
- 充電中に端末の**画面は消してOK**（wake-lock は Termux プロセスとディスプレイの両方を保つ）。
- 充電中も Android が「省電力」へ移行しないよう、第2節の除外を必ず行う。

## 4. asmr-dl 側の設定（`~/.config/asmr-dl/config.toml`）

```toml
wake_lock = true    # 起動時に termux-wake-lock（既定: true）
notify    = true    # 完了/失敗/キャンセル時の通知（既定: true）
media_scan = true   # 完了時に termux-media-scan（既定: true）
```

- `wake_lock = false` にすると、充電しながらでも Android 側の kill で止まりやすい。**長時間 DL では true のまま**。
- `notify = false` にすると、通知が来ない（ログのみ）。

## 5. 長時間ダウンロードの実行手順

1. 充電を接続
2. 第2節のバッテリー最適化除外を**毎回確認**（Android の更新でリセットされる場合がある）
3. `asmr-dl` を起動 → 画面右上 `battery: N%`（`termux-battery-status` 連携）
4. URL を貼り付け（複数可）→ Enter
5. 画面を消して**充電だけ**放置
6. 完了時に通知（`notify = true`）
7. 音楽アプリ（Poweramp / Musicolet 等）で再生確認

## 6. 途中で止まった場合の手順

| 症状 | 確認 |
|---|---|
| `downloading` のまま固まる | 端末のタスクマネージャで Termux が生きているか / `termux-wake-lock` が有効か |
| `failed: yt-dlp: ...` | `UPDATING_YTDLP.md` を参照（取得側の問題） |
| `failed: ffmpeg: ...` | `ffmpeg -version` / `ffmpeg -encoders \| grep opus` を確認 |
| 完全にログが止まる | タスクマネージャで kill されている → 第2節を再実行 |
| `output_dir` まで行かない | `ls ~/storage/shared` の権限（`termux-setup-storage` を再実行） |

## 7. よくある勘違い

- 「`termux-wake-lock` だけで安心」→ **違う**。Android 本体のバッテリー最適化で Termux プロセスが kill される（第2節必須）。
- 「充電していれば大丈夫」→ **違う**。充電中も最適化が効く。
- 「画面を開いておけば大丈夫」→ 一時的にしか効かない。長期は第2節。

## 関連

- 設計: `../arch/termux.md`
- 取得が落ちる場合: [UPDATING_YTDLP.md](./UPDATING_YTDLP.md)
- スキル: `.agent/skills/device-testing/SKILL.md`（D: Termux 系）
