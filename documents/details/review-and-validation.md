# D-1.0 照合・再点検・検証記録

以下は調査当時の記録。2026-11-22に全22件の製品判断が確定した。現行仕様と適用範囲は[TBD管理表](../rpn-calc-tbd-list.md)を参照する。


確認日：2026-11-14（日本時間）。対象は00～13の14仕様書とREADME、関連する既存Markdown。

## 1. 基準資料と優先順位

基準developは`8c960ccde4a57eed6979f8b35b646a0bceb23757`（PR #4マージ後）。作業開始時のローカルHEAD `c66ac55e43a9af9216dc83a915179240dc4cfb99`と基準developはtreeが同じであることを確認した。

| 資料 | 参照した契約・扱い |
|---|---|
| [全体設計v0.6](../rpn-calc-detailed-design.md) | 型、算術、命令効果、CLI、実装段階 |
| [IA-1.0](../rpn-calc-input-api-design.md) | 入力所有・API、CP-01～03、エラー順位の正本 |
| [SI-1.1](../rpn-calc-stack-input-detailed-design.md) | セッションの21状態遷移、編集・Liftの直交性 |
| [試験仕様](../rpn-calc-stack-input-test-cases.md) | 元fixture、SI/IA追加ケース、層の移設、保留 |
| [TBD管理表](../rpn-calc-tbd-list.md) | 現行22件、状態・暫定案・判断ゲート |
| [設計判断記録](../rpn-calc-design-decisions.md) | DD-001～044の現行・置換・過去提案の区別、本作業のDD-045～048 |
| [マニュアル照合記録](../rpn-calc-manual-review.md) | 過去のHP確認事実と残件。今回の実機観測とは扱わない |

既存文書の過去章に残るClosed表記・旧版集計・未承認提案は当時の記録として読む。現行TBD表とIAを優先し、過去記録から動作を再採用しない。外部リンク先や元添付PDFを今回再取得・再照合したとは主張しない。

## 2. 単位ごとの照合

| 単位 | 照合先 | 確認内容 |
|---|---|---|
| 00 | 全体§2/12、IA§1/3 | 3クレート、依存方向、公開/内部型、試験配置 |
| 01 | 全体§3/4.10/8、IA§4 | 全CalcError、順位、Input/Literal分離、Keep/Clear/Set |
| 02 | 全体§4.7/7.1、IA§2/3 | 幅、raw、±0、有限値、Push前の幅外検査 |
| 03 | 全体§3.2/4.4/4.6/4.11/5.1 | 4段配列式、T複製、固定領域、Snapshot、reset |
| 04 | 全体§4.8/7.1～7.4/7.7 | Y op X、u128乗算、Borrow、1補数暫定、Float flags |
| 05 | 全体§4.9/7.5/7.6 | enum所有、W/W+1回転、65bit、引数式APIとHPの差 |
| 06 | 全体§4.12、TBD-018 | DEC解釈、raw表示、F3、負0、表示とCLI I/Oの分離 |
| 07 | IA§3/4、全体§3.4/5.2 | Command全40variant、Push/Start/ReplaceX、L/Lift/flags、原子性 |
| 08 | SI§2/6、IA§3/4 | 編集文法、preview/finish、256文字、Float符号・指数・保留BS |
| 09 | SI§5/10、IA§3～5 | SI-01～21、所有、CHS分岐、全セッション取消 |
| 10 | IA§2/4、試験§9.1 | 長整数、符号・接頭辞、Onesビット列化、単独ハイフン |
| 11 | 全体§9、IA§3～5 | 行構文と実行時数値解析、pending、REPL/Batch、出力・終了コード |
| 12 | 全体§4.4、SI§6.3、IA§4 | 全領域変換、同一mode、負0、厳密性、HP WSIZE/FLOATとの差 |
| 13 | 全体§10～12、試験§7～9 | ID移設、保留、実装依存、試験と互換確認の区別 |

## 3. 再点検で修正・明確化した点

| 指摘 | 対応 |
|---|---|
| 全体§7.2に「完成数値の負入力拒否」というIA以前の注記が残る | CLIはUnsigned負数も剰余受理、Coreは完成raw厳密検査へ注記を訂正。CHSの式は維持 |
| RegisterBank::replace_allの長さ不一致のvariantが略記されている | 内部不変条件違反としてInvariantViolation、元領域保持へ具体化。全体§4.11と03を同期（DD-046） |
| Batch末尾検査でCLIがInputのprivate finishを直接呼べない | 同一radixのSetRadixをセッションへapplyする経路を明記。値/L/flags/Lift保持、ENTERなし、公開API追加なし（DD-047） |
| Coreと編集が同時に内部破損した同順位ケースの包み先が未指定 | Core→InputStateの内部検査順と包み先を固定しIAと01を同期（DD-048） |
| 旧「入力固有5種」とIAのInputErrorにInvariantViolationがある | 通常入力5種＋内部不整合の6variantとして説明。variantを削除しない |
| 新規原稿で通常Literalを永続pendingへ入れる説明があった | 通常parse→Pushと、別途ステージされたpendingの一括確定を分離。失敗後の余分なpending状態を新設しない |
| 新規原稿で1補数DIVの非0商にも範囲外剰余処理を記載した | 合法な非0除数では商が値域内であることを確認し、不要な規則を削除 |
| new/reset/SnapshotはResultを返さない | 回復可能な確保失敗とOS OOM abortの保証範囲を明記し、APIを勝手に変更しない |
| 浮動小数点の負0で一般の往復比較が符号を失う | bit比較と、Unsigned/Twosへの変換時だけ明示的な符号喪失を許す規則を分離 |

## 4. 残す判断と範囲

TBD状態はResolved 6 / Partial 11 / Open 5のまま。TBD-021の符号だけ残るFloat BS、TBD-013のHP桁境界、TBD-008の1補数互換、004/006の実機設定API・raw記憶領域等を今回の完成条件へ紛れ込ませない。各単位と13に判断時点・保留ケース・暫定採用範囲を明示した。

Floatの標準表示の指数表記閾値、REPL画面の細かなレイアウト、edition・ライブラリ版は実装記録で選択する。数値往復性・出力先・終了コード・所有境界等の保証とは分ける。これらの選択をHP互換確定へ昇格させない。

## 5. 検証結果と限界

- 00～13の14番号の重複・欠落なし。READMEから全仕様書へリンクする。
- 新規Markdown全16ファイルのUTF-8/LF、末尾改行、行末空白、表27個の列数、コードフェンス、ローカルリンク62件、既存TEST/SI/IA/TBD ID参照108箇所を検査し、すべて通過。
- 新規Command全40variant、CalcError全15variantと順序、Calculatorの公開8メソッドとHpInputSessionの公開5メソッドのシグネチャが正本と一致。SI-01～21の対応は重複・欠落なし。
- TBD管理表と全体第11章の22行が一致し、Resolved 6 / Partial 11 / Open 5を維持。
- Mermaid 11.12.0＋JSDOM 26.1.0で、新規仕様書とREADMEの全16図の構文解析が通過。
- CLI整数式を、9幅×4基数×8値×2符号×3形式の1728件で、逐次マスクと独立した全桁BigInt式の間で照合し、全件一致。
- `git diff --check`を通過。基準developとsrc/Cargo.toml/Cargo.lockの差分がないことを確認。

仕様書作成後の照合と、その後の再点検・修正を経て、上記検証を最終原稿へ再実行し、検出エラー0件を確認した。数値照合は仕様式の確認でありRust実装の試験ではない。Mermaidの画像描画・視覚的レイアウト検証、Rust製品テスト、実機試験は実施していない。今回の変更は設計資料と今後の格納先の指示だけで、製品Rust/Cargoは変更しない。
