# rdp-calc 未確定事項一覧

2026-11-21 製品仕様（TBD-021 / DD-049）：Float編集でBS後に仮数の符号だけが残る場合、bufferを `0`、Xを `+0.0` とし、Editing / MantissaのままLiftを保持する。次の数字は追記する。IA-G012（旧SI-H003）を製品受入試験に含める。HP実機との照合はTBD-017で継続する。 TBD-001・006・008・016・018・019の製品判断を含む現在の集計はResolved 13 / Partial 9 / Open 0。以下の旧集計・保留記述は決定前の記録。

文書版：v0.6 + IA-1.0/SI-1.1 / 2026-10-31 / UTF-8・LF

[詳細設計書](rdp-calc-detailed-design.md) 第11章と同じ管理表。仕様の正本は詳細設計書の各章、TBDの管理表は本ファイルとする。変更時には第11章も同期する。2026-10-31現在、Resolved 6件、Partial 11件、Open 5件（全22件）。過去ClosedはResolvedと同義で、状態名だけ統一した。Partialは資料上の事実の一部が確定し、実装契約の選択または境界確認が残る。暫定仕様はHP-16C確認済み仕様ではない。

「着手前」は該当機能の実装前を意味し、プロジェクト全体を止めるものではない。未確定のまま進める場合は表の暫定案を実装対象として明示し、暫定試験だけを合格にする。完全互換の合格条件は未確定のまま保持する。

| TBD-ID | 状態 | 内容 | 関連モジュール | 現在の暫定仕様 | 確認方法 | 確定条件 | 優先度 | 判断の期限・実装への影響 |
|---|---|---|---|---|---|---|---|---|
| TBD-001 | Resolved | 起動・リセット直後のLift | calculator,input | Disabled。初回入力はXを置換。ENTER/CLxはD、基数変更は中立、Idle BSPはCLx相当 | TEST-020～023/026/069 | 製品の初期状態と入力遷移を検証 | High | 2026-11-21確定 |
| TBD-002 | Resolved | CHSのEditing/Idle別効果とSRC-03内の矛盾 | calculator,input,arithmetic | Integerは入力終了・補数演算・SaveX・Lift E。Float Editingは符号編集/Keep、Idleは符号反転/Keep/E | 原版p.44/99/100。モード別分類を確定。UnsignedゼロのG条件はTBD-007、編集細部は017へ分離 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-003 | Resolved | LAST X対象命令の完全な分類 | calculator,command | §5.2.1で現Command全件をSave/Keep/Clearへ分類。Float CHSはKeep。HP FLOATはClear、WSIZEはSave | 原版p.100の一覧を全件照合。独自APIの契約は明示分離。TEST-034/035/070ほか | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-004 | Partial | ワード長変更の各領域の変換 | state,word,register | スタック縮小は低位保持、拡大はゼロ拡張。実機記憶領域はraw保存・境界再配置。現Vecは未対応 | 原版p.32/67。RegisterBank再設計、LAST X幅変換、HP WSIZE Commandの追加が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-005 | Resolved | 補数形式変更時の値とflags | state,word | Integerの全形式間でraw保持、数学値だけ再解釈。flags/Lift保持、LAST X履歴保存なし | 原版p.35/98～100、TEST-041。±0はビット列を保持。Float中の設定変更はTBD-006へ分離 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-006 | Resolved | 整数/Float切替 | state,number,calculator | f64優先。全4段・LAST X・全Riを厳密変換。丸めによる情報欠落時は全状態を保持して失敗 | TEST-042、IA-C010、SI-T056 | 相互変換・負0・原子性を検証。HP FLOATは製品対象外 | High | 2026-11-21確定 |
| TBD-007 | Partial | 全命令のF4/F5更新規則 | flags,arithmetic,bitwise,calculator | Appendix A更新対象分類を確定。RMDはC/G Keep。Float四則は実機でG更新 | 原版p.44/98。1補数境界、Unsigned CHS(0)、独自f64とGの統合方針が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-008 | Resolved | 1の補数の算術結果とゼロ表現 | word,arithmetic,command,CLI | 既定は算術結果の±0を+0に統一。oneszero rawで演算式のゼロ符号を保持 | TEST-018/058、W1～4の全組合せ | 2方式の結果・フラグ・切替を製品仕様で検証 | High | 2026-11-21確定 |
| TBD-009 | Resolved | ASR、n回転、回数0/負/幅以上、Carry | bitwise,calculator | ASRはUnsignedでSR同等。n=abs(X)、通常0..W/Carry0..W+1、Y対象、Drop、元X保存、0回C保持 | 原版p.47/49/97/98/100。0回は0回反復と分類から導出、実機未観測。TEST-050/051/063 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-010 | Partial | MASKL/MASKR/SB/CB/B?のキー意味と配置 | bitwise,command | HP MASKはabs(X)の単項置換、SB/CBはY対象・abs(X)番号・Drop。B?の条件スキップは対象外 | 原版p.50/51/97/100。独自引数APIとHP Commandを分離する設計が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-011 | Partial | STO/RCLリフト、レジスタ数/番号 | register,calculator | STOは値保存でLift E、RCLは旧Liftに従い条件Lift後E、いずれもLAST X Keep | 原版p.25/99/100。固定16個・最大1024の独自規則と実機の番号/記憶領域境界との差が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-012 | Partial | HPエラー番号と解除キー動作 | error,calculator,CLI | HPは任意1キーでエラー表示を消して前表示へ戻す。コアは原子性を維持 | SRC-09印刷p.96。HP番号のUI写像、解除キーの消費/再実行は未確定。プログラムError4/5は対象外 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-013 | Partial | CLI超過入力とHPキー境界の分離 | cli,rdp-input | CLI部分Resolved（CP-01）。HPキーは厳密拒否を暫定維持、W≤3初回例外等は未採用 | IA §2、SI-H001/002/008、既存H32記録 | HP初回/継続桁・DEC符号bit・flags/Liftを確認 | High | HP互換キー入力ゲート。CLI正規化は実装可 |
| TBD-014 | Partial | LJ、DBL×/DBL÷/DBLR、実機WSIZE | arithmetic,calculator,command | HP WSIZE: abs(X)、0→64、Drop、元X保存。LJ非0: 左寄せ値をY、移動数をX、C/G Keep | 原版p.32/47/98/100。LJ(0)、DBL全結果配置/境界、HP WSIZEの実装APIを残す | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-015 | Resolved | プログラム記録・分岐・制限・実行API | CLI,docs | 対象外として確定。ProgramEngine等の候補型/APIを削除。自動化は外部スクリプト→CLI | ユーザー方針SRC-10。TEST-064を外部CLI自動化へ変更。HPプログラム互換試験は不要 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Low | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-016 | Resolved | Floatの精度・数値範囲 | arithmetic,format | Rustのf64を基準とする。有限のsubnormalを受理し、非有限値は拒否 | TEST-067 | HP-16Cとの精度・範囲の一致は要求しない。フラグはTBD-007で扱う | Medium | 2026-11-21 製品判断 |
| TBD-017 | Partial | 浮動小数点編集・Backspace・指数符号 | input | Float CHSは編集継続・LAST X Keep、BSP IdleはCLx相当へ修正 | 原版p.25/99/100。仮数/指数の削除境界と入力中最終桁削除の全状態が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-018 | Resolved | 表示桁・ゼロ埋め | format,CLI | 非DEC整数はraw、F3で幅に合わせてゼロ埋め。DECは数学値、Floatは最短往復表現。表示窓なし | TEST-043 | 全基数・端数幅・負0・Float往復を検証 | Low | 2026-11-21確定 |
| TBD-019 | Resolved | レジスタ命令の対応範囲 | register,command | 直接指定のSTO/RCLのみ | 命令表と直接STO/RCLの受入試験 | レジスタ演算・Iレジスタ・間接指定は対象外。記憶領域と番号体系はTBD-011で扱う | Medium | 2026-11-21 製品判断 |
| TBD-020 | Partial | HP-16C機能全体の到達範囲 | command,docs | 内蔵プログラミングを対象外、ABSをHP標準へ訂正。外部CLI自動化を正式用途とする | SRC-10/原版p.44。非プログラム命令の必須/将来/対象外の完全な棚卸しは継続 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-021 | Resolved | Float BSで符号だけ残る境界 | rdp-input | 正0へ正規化し編集・Liftを保持 | DD-049、IA-G012（SI-H003） | 製品動作確定。実機照合は017 | High | 2026-11-21確定 |
| TBD-022 | Resolved | エラー優先順位とvariant | core,rdp-input,cli | CP-03、IA §4の順位・依存ガード・型分離を採用 | IAとIA-C/IA-G試験 | 設計確定。実装時variant/payloadと全状態を検査 | Medium | 複合原因試験を必須化。実機番号は012に残る |

## 実装開始時に先に選択する項目

基本入力ではTBD-013、幅変更ではTBD-004、n回転ではTBD-009について「資料で確定する／本書の暫定動作で進める／当該命令を未対応にする」を実装記録へ残す。TBDだからといってWordSize、固定スタック、原子性、Unsigned演算、CLI構文の作業を止めない。

TBD-003はClosed、TBD-007の残件は純粋演算関数内で差し替えられる。TBD-010/014は新しいCommandが必要になる可能性があり、現APIで実機命令を代用してはいけない。TBD-015は対象外としてClosedであり、将来実装ゲートにはしない。

## クローズ手順

1. 版と印刷ページを明記した一次資料または実機の設定・キー列・結果を保存する。
2. 期待X/Y/Z/T、LAST X、F4/F5、EntryPhase、StackLiftを必要な範囲で記録する。
3. 暫定仕様との差分、影響する関数とテストを列挙する。
4. 詳細設計書の該当節・命令表・状態表・図・試験表を一緒に更新する。
5. 外部確認済みと独自仕様を区別したままClosedとする。エミュレーターだけで確認した場合はその根拠と限界を残す。

TBDを閉じる際の追記欄：状態、確認日、資料版/ページ、観測手順、決定内容、関連TEST-ID、変更した設計判断ID。


## v0.5 照合履歴

確定根拠・文書導出・残件は[マニュアル照合記録](rdp-calc-manual-review.md)を参照。Closedは実装済みを意味しない。TBD-002の数値フラグ境界は007、Float編集細部は017へ分離した。TBD-005はInteger内の形式変更を確定し、Float中の設定変更は006で扱う。元のIDを削除せず追跡を継続する。


## SI-1.0 更新台帳（過去記録、現状は冒頭管理表）（2026-10-31）

[詳細補遺](rdp-calc-stack-input-detailed-design.md) §9と[試験仕様](rdp-calc-stack-input-test-cases.md)を参照。Resolvedは仕様根拠または製品判断の解決であり実装済みではない。既存5件のClosed→Resolvedは表記統一だけで、今回新たに全体解決したTBDはない。

| 項目 | 状態 | 現状・調査結果 | 推奨案 | 判断に必要な情報 | 実装への影響 |
|---|---|---|---|---|---|
| 001 通常遷移の部分項目 | Resolved | ENTER自身のliftと次回抑止、CLx、基数中立はV5と今回の本文で整合 | SI表を使用 | 起動/resetのHP状態だけ残る | 親001はPartial、基本遷移は着手可 |
| 002/003 | Resolved | モード別CHSとLAST X分類を継承、CE本文とも整合 | 分類表維持 | 符号0/編集細部は017/021 | 全体Resolvedを維持 |
| 011 基本STO/RCLの部分項目 | Resolved | 条件lift/保存元/KeepLを具体化 | SI-T020～022 | 物理領域と番号体系が残る | 親011はPartial |
| 013 | Partial | H32脚注で初回幅外桁→0を確認。DEC符号bit到達との整合も要検討 | CP-01、キー入力とCLI/Pushを分離 | 桁超過後の全状態、DEC境界、採否 | HP互換キー入力ゲート、厳密入力は先行可 |
| 017 | Partial | 一般BS分類は確認。負数最終桁と指数削除は細部未確認 | 一般編集を維持、穴を021へ分離 | 原版画像/実機の削除列 | Float完全編集の互換宣言前に必要 |
| 021 | Open | -1から1字削除すると文法外の符号だけになる | CP-02で0へ正規化を提案 | 符号0保持の製品判断とHP観測 | 該当BS境界のみ着手前レビュー |
| 022 | Open | 複数原因Errの順序、精度欠落のvariantが不明 | CP-03で順序を明文化 | API利用者のvariant保証要件 | 固定variant試験は保留、原子性は確定 |
| 004/006/014 | Partial | 直接設定APIとHP WSIZE/FLOATは異なる | 別Command/記憶表現設計を継続 | raw領域、LAST X幅、逆変換丸め | HP互換設定のブロッカー。全体開発停止ではない |
| 012 | Partial | コア原子性は製品確定、実機画面解除キーは残件 | コアにError状態を増やさない | 解除キーの消費/再実行 | 互換UIのゲート |

変更提案を採用したら、SI本文・SIテスト・V5の該当契約・本表とV5第11章を同時更新する。今回、厳密拒否/Push/f64/原子性/モジュール所有関係は変更していない。

## IA-1.0 更新台帳（2026-10-31）

CP-01/CP-02/CP-03はResolved。013はCLI部分だけResolved、親013はHPキー未解決のためPartial。021はCLI問題を切り離しOpen維持。022は順位・精度欠落・重複点variant確定によりResolved。現行22件の集計はResolved 6 / Partial 11 / Open 5。過去台帳の提案・集計は当時の履歴として保持する。
