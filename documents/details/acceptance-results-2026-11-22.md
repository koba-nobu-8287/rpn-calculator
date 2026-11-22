# PC版 RPN電卓 製品受入結果（2026-11-22）

**判定：合格。** PR #18 マージ後の [`develop` のコミット `f50bca1`](https://github.com/koba-nobu-8287/rpn-calculator/commit/f50bca1bf519790df60f9c89da786d6a3b75e45a) を対象に、PC版の製品受入仕様195件を確認した。Debug・Releaseともに167件のRustテストが成功し、失敗・無視は各0件だった。

## 対象と照合方法

対象は `rpn-core`、`rpn-input`、`rpn-cli`。検証環境はWindows（`x86_64-pc-windows-msvc`）、rustc 1.94.1、Cargo 1.94.1。実行した作業コピーのGitツリーは `dd63d81d6e44eed07bec12bde9b07a9a6738ab61`。PR #18の先頭とマージ後の対象コミットとのファイル差分は0件だった。

| 受入仕様 | 対象ID数 | 判定 |
|---|---:|---|
| TEST　演算・状態・CLIの製品仕様 | 69 | 合格 |
| SI-T　スタックと入力の状態遷移 | 76 | 合格 |
| IA-L　CLI入力 | 22 | 合格 |
| IA-C　Core API | 16 | 合格 |
| IA-G　GUI相当のキー入力と状態保持 | 12 | 合格 |
| **合計** | **195** | **合格** |

[製品受入試験対応表](product-acceptance.md)の195個の製品IDからテスト関数への参照232件を、今回のDebug・Release双方の実行結果と照合した。全参照先で成功を確認した。1つのテストが複数の仕様IDを検証する場合と、1つの仕様IDを複数のテストで検証する場合がある。

## 実行結果

| コマンド | 結果 |
|---|---|
| `cargo test --workspace --offline` | 167成功、0失敗、0無視 |
| `cargo test --workspace --release --offline` | 167成功、0失敗、0無視 |
| `cargo fmt --all -- --check` | 成功 |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | 成功 |

HP実機比較用のTEST-065/066/071/072は製品判定の対象外。旧SI-H003/004の製品要件はIA-G012/004で各1回計上した。その他の実機比較項目の扱いは[対応表の末尾](product-acceptance.md#hp実機比較と旧id)に記載した。
