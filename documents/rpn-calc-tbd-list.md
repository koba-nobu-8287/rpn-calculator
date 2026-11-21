# rpn-calc 未確定事項一覧

2026-11-22 現行製品の22件はすべてResolved（Partial 0、Open 0）。下表は製品仕様の決定を記録する。HP資料・実機との一致や試験実績は各欄の根拠に従う。

文書版：v0.6 + IA-1.0/SI-1.1 / 2026-10-31 / UTF-8・LF

本表を管理元とし、[詳細設計書](rpn-calc-detailed-design.md) 第11章と同期する。

| TBD-ID | 状態 | 内容 | 関連モジュール | 製品仕様 | 確認方法 | 確定条件 | 優先度 | 判断の期限・実装への影響 |
|---|---|---|---|---|---|---|---|---|
| TBD-001 | Resolved | 起動・リセット直後のLift | calculator,input | Disabled。初回入力はXを置換。ENTER/CLxはD、基数変更は中立、Idle BSPはCLx相当 | TEST-020～023/026/069 | 製品の初期状態と入力遷移を検証 | High | 2026-11-21確定 |
| TBD-002 | Resolved | CHSのEditing/Idle別効果とSRC-03内の矛盾 | calculator,input,arithmetic | Integerは入力終了・補数演算・SaveX・Lift E。Float Editingは符号編集/Keep、Idleは符号反転/Keep/E | 原版p.44/99/100。モード別分類を確定。UnsignedゼロのG条件はTBD-007、編集細部は017へ分離 | 製品仕様は確定。根拠は照合記録を参照 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-003 | Resolved | LAST X対象命令の完全な分類 | calculator,command | §5.2.1で現Command全件をSave/Keep/Clearへ分類。Float CHSはKeep。HP FLOATはClear、WSIZEはSave | 原版p.100の一覧を全件照合。独自APIの契約は明示分離。TEST-034/035/070ほか | 製品仕様は確定。根拠は照合記録を参照 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-004 | Resolved | ワード長変更の各領域の変換 | state,word,register | 整数のWS（CLI `wsize N`）は4段・LAST X・全Riを一括変換。縮小は下位保持、拡大はゼロ拡張。番号・個数を保持 | SI-T053/075 | W16→W8→W16でR0=0x5678は0x0078 | High | 2026-11-22 製品判断 |
| TBD-005 | Resolved | 補数形式変更時の値とflags | state,word | Integerの全形式間でraw保持、数学値だけ再解釈。flags/Lift保持、LAST X履歴保存なし | 原版p.35/98～100、TEST-041。±0はビット列を保持。Float中の設定変更はTBD-006へ分離 | 製品仕様は確定。根拠は照合記録を参照 | Medium | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-006 | Resolved | 整数/Float切替 | state,number,calculator | f64優先。全4段・LAST X・全Riを厳密変換。丸めによる情報欠落時は全状態を保持して失敗 | TEST-042、IA-C010、SI-T056 | 相互変換・負0・原子性を検証。HP FLOATは製品対象外 | High | 2026-11-21確定 |
| TBD-007 | Resolved | 全命令のF4/F5更新規則 | flags,arithmetic,bitwise,calculator | 1補数のADDのCは補正前の桁上がり、SUBのCはrawの借り。Gは数学値域で判定し、同一入力で両oneszeroモードのflagsは一致。Unsigned CHSはC保持、Gは入力が非0のとき1。有限f64演算はC/G保持 | 仕様04・05、TEST-015～019/044～046/068 | 失敗時は全状態保持。DIV・RMD・SQRTは命令表に従う | High | 2026-11-22 製品判断 |
| TBD-008 | Resolved | 1の補数の算術結果とゼロ表現 | word,arithmetic,command,CLI | 既定は算術結果の±0を+0に統一。oneszero rawで演算式のゼロ符号を保持 | TEST-018/058、W1～4の全組合せ | 2方式の結果・フラグ・切替を製品仕様で検証 | High | 2026-11-21確定 |
| TBD-009 | Resolved | ASR、n回転、回数0/負/幅以上、Carry | bitwise,calculator | ASRはUnsignedでSR同等。n=abs(X)、通常0..W/Carry0..W+1、Y対象、Drop、元X保存、0回C保持 | 原版p.47/49/97/98/100。0回は0回反復と分類から導出、実機未観測。TEST-050/051/063 | 製品仕様は確定。根拠は照合記録を参照 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-010 | Resolved | MASKL/MASKR/SB/CB/B?のキー意味と配置 | bitwise,command | マスク長・bit番号は明示引数。セット／クリアはXが対象、照会はXを保持してboolを返す | 仕様05、TEST-052 | HP式スタック引数・条件スキップは対象外 | Medium | 2026-11-22 製品判断 |
| TBD-011 | Resolved | STO/RCLリフト、レジスタ数/番号 | register,calculator | 既定16個、生成時1～1024個、0始まり。直接STOはXを保存、RCLは旧Liftで条件リフト。両者ともLAST X保持、成功後Lift有効 | SI-T020～022/065 | 範囲外はInvalidRegister(n)、編集を含む状態保持 | Medium | 2026-11-22 製品判断 |
| TBD-012 | Resolved | HPエラー番号と解除キー動作 | error,calculator,CLI | 失敗した操作の直前の状態を保持。CLIは診断し、次のキーを通常処理。将来GUIは通知ダイアログで表示し、電卓への次入力を消費しない | SI-T061～063、IA-G008 | HP番号・HP式エラー解除キーは対象外 | Medium | 2026-11-22 製品判断 |
| TBD-013 | Resolved | CLI超過入力とHPキー境界の分離 | cli,rpn-input | CLI完成整数は下位Wビット。キーは初回・継続ともDECの数学値域または非DECのraw幅を厳密検査 | SI-T045～052、IA-G003 | 幅外はCalculation(IntegerOutOfRange)、入力・Lift・flagsを含む全状態保持 | High | 2026-11-22 製品判断 |
| TBD-014 | Resolved | LJ、DBL×/DBL÷/DBLR、実機WSIZE | arithmetic,calculator,command | HP式WSIZEは対象外。幅は `wsize N` で指定。LJは今回対象外、DBL×・DBL÷・DBLRは将来拡張 | 命令表§7.6・CLIの未対応語 | 倍長演算のAPI・結果配置は拡張時に設計 | Medium | 2026-11-22 製品判断 |
| TBD-015 | Resolved | プログラム記録・分岐・制限・実行API | CLI,docs | 対象外として確定。ProgramEngine等の候補型/APIを削除。自動化は外部スクリプト→CLI | ユーザー方針SRC-10。TEST-064を外部CLI自動化へ変更。HPプログラム互換試験は不要 | 製品仕様は確定。根拠は照合記録を参照 | Low | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-016 | Resolved | Floatの精度・数値範囲 | arithmetic,format | Rustのf64を基準とする。有限のsubnormalを受理し、非有限値は拒否 | TEST-067 | HP-16Cとの精度・範囲の一致は要求しない。フラグはTBD-007で扱う | Medium | 2026-11-21 製品判断 |
| TBD-017 | Resolved | 浮動小数点編集・Backspace・指数符号 | input | Float編集時のCHSは現在の仮数／指数の符号を反転。BSは末尾1文字を削除。未完成指数は仮数を表示し、確定時IncompleteEntry | SI-T027～040/073、IA-G012 | −1からBSで正0・編集継続・Lift保持。Idle BSはCLx | Medium | 2026-11-22 製品判断 |
| TBD-018 | Resolved | 表示桁・ゼロ埋め | format,CLI | 非DEC整数はraw、F3で幅に合わせてゼロ埋め。DECは数学値、Floatは最短往復表現。表示窓なし | TEST-043 | 全基数・端数幅・負0・Float往復を検証 | Low | 2026-11-21確定 |
| TBD-019 | Resolved | レジスタ命令の対応範囲 | register,command | 直接指定のSTO/RCLのみ | 命令表と直接STO/RCLの受入試験 | レジスタ演算・Iレジスタ・間接指定は対象外。記憶領域と番号体系はTBD-011で扱う | Medium | 2026-11-21 製品判断 |
| TBD-020 | Resolved | HP-16C機能全体の到達範囲 | command,docs | 今回の範囲は現行Core・共通入力・対話／ワンショットCLI。倍長3命令のみ将来の命令拡張候補 | 全体§1.1・§7.6、仕様07/11 | HP固有設定・スタック引数・LJ・間接指定・レジスタ演算・内蔵プログラムは今回対象外。自動化は外部CLI | Medium | 2026-11-22 製品判断 |
| TBD-021 | Resolved | Float BSで符号だけ残る境界 | rpn-input | 正0へ正規化し編集・Liftを保持 | DD-049、IA-G012（SI-H003） | 製品動作確定。編集全般は017 | High | 2026-11-21確定 |
| TBD-022 | Resolved | エラー優先順位とvariant | core,rpn-input,cli | CP-03、IA §4の順位・依存ガード・型分離を採用 | IAとIA-C/IA-G試験 | 設計確定。実装時variant/payloadと全状態を検査 | Medium | 複合原因試験を必須化。HP番号は012で対象外 |

## クローズ手順

1. 版と印刷ページを明記した一次資料または実機の設定・キー列・結果を保存する。
2. 期待X/Y/Z/T、LAST X、F4/F5、EntryPhase、StackLiftを必要な範囲で記録する。
3. 暫定仕様との差分、影響する関数とテストを列挙する。
4. 詳細設計書の該当節・命令表・状態表・図・試験表を一緒に更新する。
5. 外部確認済みと独自仕様を区別したままClosedとする。エミュレーターだけで確認した場合はその根拠と限界を残す。

TBDを閉じる際の追記欄：状態、確認日、資料版/ページ、観測手順、決定内容、関連TEST-ID、変更した設計判断ID。


## v0.5 照合履歴

確定根拠・文書導出・残件は[マニュアル照合記録](rpn-calc-manual-review.md)を参照。Closedは実装済みを意味しない。TBD-002の数値フラグ境界は007、Float編集細部は017へ分離した。TBD-005はInteger内の形式変更を確定し、Float中の設定変更は006で扱う。元のIDを削除せず追跡を継続する。


## SI-1.0 更新台帳（過去記録、現状は冒頭管理表）（2026-10-31）

[詳細補遺](rpn-calc-stack-input-detailed-design.md) §9と[試験仕様](rpn-calc-stack-input-test-cases.md)を参照。Resolvedは仕様根拠または製品判断の解決であり実装済みではない。既存5件のClosed→Resolvedは表記統一だけで、今回新たに全体解決したTBDはない。

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

## IA-1.0 更新台帳（2026-10-31時点の履歴）

CP-01/CP-02/CP-03はResolved。013はCLI部分だけResolved、親013はHPキー未解決のためPartial。021はCLI問題を切り離しOpen維持。022は順位・精度欠落・重複点variant確定によりResolved。現行22件の集計はResolved 6 / Partial 11 / Open 5。過去台帳の提案・集計は当時の履歴として保持する。
