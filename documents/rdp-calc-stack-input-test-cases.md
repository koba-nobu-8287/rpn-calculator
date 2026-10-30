# rdp-calc スタック・入力 単体テスト仕様

版：SI-1.0 / 2026-10-31 / 基準develop：`25bb2dea3fcb3162868e22cca5b5334c8375f4d6`

対応：[詳細設計](rdp-calc-stack-input-detailed-design.md)、[v0.5](rdp-calc-detailed-design.md)。本書は実行結果ではなく、実装時のfixture仕様。現リポジトリのRustはHello worldのみで既存の実行可能なテストはない。既存TEST番号はv0.5の試験仕様を指す。

## 1. 共通fixtureと表の読み方

- 全スタックは`[X,Y,Z,T]`順。整数値は特記の`0x`以外は十進数。負値はWordConfigでrawへ符号化する。標準設定はInteger、W16、TwosComplement、DEC、16記憶レジスタ（全て0）、LAST X=9。
- `I_D/I_E`はIdle・空buffer・Mantissa、`E_D("12")`などはEditing・指定buffer・Mantissa。`/Exp`はExponent。Xは必ずbufferのpreviewと一致するfixtureにする。
- flags初期値は`F0..F3=[1,0,1,0], C=1,G=1`。`K`は全6flags保持、`C0G0`等はF0..3保持しC/Gだけ指定更新。各KケースはC/G=00/01/10/11でも再実行する。
- 表の期待値は**最後の成功または失敗コマンド直後**。失敗が途中にある操作列は`execute`を個別に呼んで意図的に継続する。`execute_all`の停止試験は別行。
- `digit("12")`はInputDigit('1'),InputDigit('2')。`Push(12)`とは異なる。`BASE`はSetRadix、`WS`はSetWordSize、`MODE`はSetNumericMode。`+,-,*,/,RMD`は対応するCommand。`R↓/R↑/X<>Y`はRollDown/RollUp/SwapXY。
- 期待状態に書かない設定・記憶領域は全保持。Lは全行明示。`same`は入力・Lift・buffer・part・全数値・設定・記憶領域・flagsすべてを操作直前と同一にする。Floatは必要時`to_bits()`比較。
- 優先度P0：基本状態機械・破壊防止、P1：編集／設定／境界、P2：拡張・互換保留。
- 根拠H22/H25/H32/H99/CE/V5-Hは詳細設計§1.1。`V5`は製品契約（HP未確認を含む）、`導出`は規則合成。HP未確認の部分を全体の互換合格へ含めない。

公開restore APIをテストのために追加しない。`stack`/`input`のモジュール内単体テストではprivate fixtureを構築可能。公開API結合テストではnew/with_configとCommand列で到達状態を作り、セットアップ後の全Snapshotを確認してから本操作を開始する。

## 2. スタック・入力開始・履歴

| ID・対象 | 初期スタック／L | 初期入力・設定差分 | 操作列 | 期待スタック／L | 期待入力・設定差分 | flags | 根拠 | 優先度 |
|---|---|---|---|---|---|---|---|---|
| SI-T001 ENTER | [1,2,3,4]/9 | I_E | ENTER | [1,1,2,3]/9 | I_D | K | H22/H25 | P0 |
| SI-T002 連続ENTER | [1,2,3,4]/9 | I_D | ENTER×3 | [1,1,1,1]/9 | I_D | K | H22導出 | P0 |
| SI-T003 編集ENTER | [12,2,3,4]/9 | E_E("12") | ENTER | [12,12,2,3]/9 | I_D | K | H22/V5 | P0 |
| SI-T004 ENTER後数字 | [5,2,3,4]/9 | E_D("5") | ENTER, digit("3") | [3,5,2,3]/9 | E_D("3") | K | H22/H25 | P0 |
| SI-T005 Eから初回数字 | [8,2,3,4]/9 | I_E | digit("12") | [12,8,2,3]/9 | E_E("12") | K | H22/V5導出 | P0 |
| SI-T006 Dから初回数字 | [8,2,3,4]/9 | I_D | digit("12") | [12,2,3,4]/9 | E_D("12") | K | H25/V5導出 | P0 |
| SI-T007 二項Add | [3,5,2,4]/9 | E_D("3") | + | [8,2,4,4]/3 | I_E | C0G0 | H22、V5 §7 | P0 |
| SI-T008 非可換Subtract | [3,5,2,4]/9 | I_D | - | [2,2,4,4]/3 | I_E | C0G0 | V5 §7、H22 | P0 |
| SI-T009 Tの反復複製 | [2,10,20,30]/9 | I_E | +,+,+ | [62,30,30,30]/32 | I_E | C0G0 | H22導出 | P0 |
| SI-T010 演算直後数字 | [3,5,2,4]/9 | I_D | +,digit("7") | [7,8,2,4]/3 | E_E("7") | C0G0 | H22/V5 | P0 |
| SI-T011 単項ABS | [-5,2,3,4]/9 | I_D | ABS | [5,2,3,4]/-5 | I_E | C1G0 | V5 §7、V5-H p.44/98/100 | P0 |
| SI-T012 R↓ | [1,2,3,4]/9 | E_D("1") | R↓ | [2,3,4,1]/9 | I_E | K | V5-H p.23/99/100 | P0 |
| SI-T013 R↑ | [1,2,3,4]/9 | E_E("1") | R↑ | [4,1,2,3]/9 | I_E | K | V5-H p.23/99/100 | P0 |
| SI-T014 交換 | [1,2,3,4]/9 | E_D("1") | X<>Y | [2,1,3,4]/9 | I_E | K | V5-H p.23/99/100 | P0 |
| SI-T015 循環往復 | [1,2,3,4]/9 | I_E | R↓×4,R↑×4,X<>Y×2 | [1,2,3,4]/9 | I_E | K | V5-H導出 | P1 |
| SI-T016 CLx編集中 | [12,2,3,4]/9 | E_E("12") | CLx,digit("7") | [7,2,3,4]/9 | E_D("7") | K | H25/H99 | P0 |
| SI-T017 CLx未完成 | [1,2,3,4]/9 | Float、E_E("1e-")/Exp | CLx | [0,2,3,4]/9 | I_D、Float | K | V5、CE p.17 | P0 |
| SI-T018 LAST X有効Lift | [3,5,2,4]/9 | I_E | LAST X | [9,3,5,2]/9 | I_E | K | V5-H p.23–25/100 | P0 |
| SI-T019 LAST X無効Lift | [3,5,2,4]/9 | I_D | LAST X,LAST X | [9,9,5,2]/9 | I_E | K | V5-H導出 | P0 |
| SI-T020 STO | [12,2,3,4]/9 | E_D("12") | STO 0 | [12,2,3,4]/9 | I_E、R0=12 | K | V5-H p.66/100 | P0 |
| SI-T021 RCL D | [12,2,3,4]/9 | E_D("12"),R0=7 | RCL 0 | [7,2,3,4]/9 | I_E、R0=7 | K | H25、V5-H p.66/100 | P0 |
| SI-T022 RCL E | [12,2,3,4]/9 | E_E("12"),R0=7 | RCL 0 | [7,12,2,3]/9 | I_E、R0=7 | K | V5-H導出 | P0 |
| SI-T023 Push区別 | [1,2,3,4]/9 | I_D | Push(5),Push(3),+ | [8,2,3,3]/3 | I_E | C0G0 | V5独自 | P0 |
| SI-T024 Push編集中拒否 | [12,2,3,4]/9 | E_D("12") | Push(3) | same/9 | E_D("12"),InputInProgress | K | V5独自 | P0 |

## 3. CHS・点・削除・不完全入力

| ID・対象 | 初期スタック／L | 初期入力・設定差分 | 操作列 | 期待スタック／L | 期待入力・設定差分 | flags | 根拠 | 優先度 |
|---|---|---|---|---|---|---|---|---|
| SI-T025 Integer CHS | [5,2,3,4]/9 | E_D("5") | CHS,digit("2") | [2,-5,2,3]/5 | E_E("2") | C1G0 | V5-H p.44/99/100 | P0 |
| SI-T026 Integer連続CHS | [5,2,3,4]/9 | E_E("5") | CHS,CHS | [5,2,3,4]/-5 | I_E | C1G0 | V5-H導出 | P0 |
| SI-T027 Float編集CHS | [5,2,3,4]/9 | Float、E_D("5") | CHS,digit("2") | [-52,2,3,4]/9 | E_D("-52") | K | CE p.58、V5 | P0 |
| SI-T028 Float連続CHS | [5,2,3,4]/9 | Float、E_E("5") | CHS,CHS | [5,2,3,4]/9 | E_E("5") | K | CE p.58、V5導出 | P0 |
| SI-T029 Float Idle CHS | [5,2,3,4]/9 | Float、I_D | CHS | [-5,2,3,4]/9 | I_E | K | V5-H/CE p.100導出 | P0 |
| SI-T030 Float負0 | [0,2,3,4]/9 | Float、E_D("0") | CHS | [-0.0,2,3,4]/9 | E_D("-0") | K | V5 f64編集、HP未確認 | P1 |
| SI-T031 点から開始 | [8,2,3,4]/9 | Float、I_E | 点,digit("5") | [0.5,8,2,3]/9 | E_E("0.5") | K | V5暫定、017 | P1 |
| SI-T032 重複点 | [1.2,2,3,4]/9 | Float、E_D("1.2") | 点 | same/9 | E_D("1.2"),Err | K | V5文法、variantはCP-03 | P1 |
| SI-T033 Integer点 | [1,2,3,4]/9 | E_D("1") | 点 | same/9 | E_D("1"),NumericModeMismatch | K | V5 | P1 |
| SI-T034 指数と符号 | [1,2,3,4]/9 | Float、E_D("1") | EEX,digit("2"),CHS | [0.01,2,3,4]/9 | E_D("1e-2")/Exp | K | CE p.58、V5、値はf64参照のto_bits | P1 |
| SI-T035 指数のBS | [0.01,2,3,4]/9 | Float、E_E("1e-2")/Exp | BS,BS,BS | [1,2,3,4]/9 | E_E("1")/Mantissa | K | V5文字削除導出、017未確認 | P1 |
| SI-T036 最終桁BS | [1,2,3,4]/9 | E_E("1") | BS,digit("2") | [2,2,3,4]/9 | E_E("02") | K | V5暫定。0をバッファに残す | P1 |
| SI-T037 Idle BS | [5,2,3,4]/9 | I_E | BS | [0,2,3,4]/9 | I_D | K | H99、CE p.17 | P0 |
| SI-T038 Idle EEX | [5,2,3,4]/9 | Float、I_E | EEX | same/9 | I_E,IncompleteEntry | K | V5 | P1 |
| SI-T039 不完全ENTER | [1,2,3,4]/9 | Float、E_E("1e")/Exp | ENTER | same/9 | E_E("1e")/Exp,IncompleteEntry | K | V5、HP細部未確認 | P0 |
| SI-T040 不完全操作 | [1,2,3,4]/9 | Float、E_D("1e-")/Exp | 各々：R↓,R↑,X<>Y,+,STO0,RCL0,LAST X,BASE DEC,MODE Float | 各試行same/9 | E_D("1e-")/Exp,IncompleteEntry | K | V5原子性、各命令独立実行 | P0 |

## 4. 基数・幅・入力上限・モード

| ID・対象 | 初期スタック／L | 初期入力・設定差分 | 操作列 | 期待スタック／L | 期待入力・設定差分 | flags | 根拠 | 優先度 |
|---|---|---|---|---|---|---|---|---|
| SI-T041 基数切替D | [10,2,3,4]/9 | HEX、E_D("A") | BASE DEC,digit("2") | [2,2,3,4]/9 | DEC、E_D("2") | K | H99、CE p.100、V5導出 | P0 |
| SI-T042 基数切替E | [10,2,3,4]/9 | HEX、E_E("A") | BASE DEC,digit("2") | [2,10,2,3]/9 | DEC、E_E("2") | K | 同上、履歴差 | P0 |
| SI-T043 同基数指定 | [12,2,3,4]/9 | E_D("12") | BASE DEC,digit("3") | [3,2,3,4]/9 | E_D("3") | K | V5終了規則から導出 | P1 |
| SI-T044 全基数往復 | [255,2,3,4]/9 | HEX、I_E | BASE BIN,BASE OCT,BASE DEC,BASE HEX | [255,2,3,4]/9 | HEX、I_E、全raw保持 | K | CE p.100、V5 | P1 |
| SI-T045 初回不正桁 | [5,2,3,4]/9 | BIN、I_E | digit("2") | same/9 | I_E,InvalidDigit('2',Binary) | K | V5厳密拒否、HP応答未確認 | P0 |
| SI-T046 編集不正桁 | [7,2,3,4]/9 | OCT、E_D("7") | digit("8") | same/9 | E_D("7"),InvalidDigit('8',Octal) | K | V5 | P1 |
| SI-T047 HEX小文字 | [5,2,3,4]/9 | HEX、I_D | digit("af") | [175,2,3,4]/9 | E_D("AF") | K | V5独自ASCII契約 | P1 |
| SI-T048 W8最大raw | [15,2,3,4]/9 | W8、HEX、E_D("F") | digit("F") | [0xFF,2,3,4]/9 | E_D("FF")、数学値−1 | K | V5 raw契約、H32 | P1 |
| SI-T049 W8超過 | [0xFF,2,3,4]/9 | W8、HEX、E_E("FF") | digit("F") | same/9 | E_E("FF"),IntegerOutOfRange | K | V5厳密入力、013 | P0 |
| SI-T050 W1初回幅外 | [1,0,0,0]/0 | W1、Unsigned、HEX、I_E | digit("2") | same/0 | I_E,IntegerOutOfRange | K | **V5現契約のみ**、HPは差ありCP-01 | P1 |
| SI-T051 256文字 | [0,2,3,4]/9 | E_D(0を255字) | digit("0") | [0,2,3,4]/9 | E_D(0を256字) | K | V5上限 | P1 |
| SI-T052 257文字拒否 | [0,2,3,4]/9 | E_E(0を256字) | digit("0") | same/9 | buffer256字、InputTooLong | K | V5上限 | P1 |
| SI-T053 幅縮小／拡大 | [0x1234,0xFF,3,4]/0x1234 | HEX、E_E("1234"),R0=0x5678 | WS8,WS16 | [0x34,0xFF,3,4]/0x34 | I_E,W16,R0=0x78 | K | V5直接API暫定、H32のスタックのみ一致 | P1 |
| SI-T054 符号形式変更 | [0xFF,2,3,4]/9 | W8、HEX、E_D("FF") | SetSignedMode(Unsigned) | [0xFF,2,3,4]/9 | I_D、Unsigned、X数学値255 | K | V5-H p.35/100 | P1 |
| SI-T055 Integer→Float | [5,2,3,4]/9 | HEX、E_D("5"),R0=7 | MODE Float | [5.0,2.0,3.0,4.0]/9.0 | Float、DEC、I_D、R0=7.0 | K | V5独自厳密変換 | P1 |
| SI-T056 変換全体拒否 | [5,2,3,4]/9 | W64 Unsigned、E_E("5"),R0=9007199254740993 | MODE Float | same/9 | Integer、E_E("5"),Err（variant未確定） | K | V5厳密変換、非表現可能なRi | P0 |
| SI-T057 Float→Integer拒否 | [1.5,2,3,4]/9 | Float、E_D("1.5") | MODE Integer | same/9 | Float、E_D("1.5"),IntegerOutOfRange | K | V5厳密変換、小数切捨て禁止 | P0 |
| SI-T058 Floatで基数 | [5,2,3,4]/9 | Float、E_E("5") | BASE HEX | same/9 | Float、E_E("5"),NumericModeMismatch | K | V5 API差、CE p.59とは別 | P1 |
| SI-T059 Floatで同基数 | [5,2,3,4]/9 | Float、E_D("5") | BASE DEC | [5,2,3,4]/9 | Float、I_D | K | V5 | P1 |
| SI-T060 同mode | [10,2,3,4]/9 | HEX、E_E("A") | MODE Integer | [10,2,3,4]/9 | HEX、I_E | K | V5同mode契約 | P1 |

SI-T056は有限f64になっても精度が失われるケース。V5 §8はこの場合のvariantを明示しないためCP-03で決める。推奨はIntegerOutOfRange（厳密変換の情報欠落）。SI-T057はV5の整数への厳密変換範囲エラー。全状態不変は両方で必須。

## 5. エラー・独自API・全状態検証

| ID・対象 | 初期スタック／L | 初期入力・設定差分 | 操作列 | 期待スタック／L | 期待入力・設定差分 | flags | 根拠 | 優先度 |
|---|---|---|---|---|---|---|---|---|
| SI-T061 除算失敗 | [0,5,3,4]/9 | E_E("0") | / | same/9 | E_E("0"),DivisionByZero | K | V5原子性 | P0 |
| SI-T062 エラー直後編集 | [0,5,3,4]/9 | E_E("0") | /失敗,digit("2") | [2,5,3,4]/9 | E_E("02") | K | V5コア契約、HP解除キーは012 | P0 |
| SI-T063 解除後新規入力 | [0,5,3,4]/9 | E_E("0") | /失敗,CLx,digit("2") | [2,5,3,4]/9 | E_D("2") | K | V5、H25 | P0 |
| SI-T064 DomainError | [-1,2,3,4]/9 | Float、E_D("-1") | SQRT | same/9 | E_D("-1"),DomainError | K | V5原子性 | P0 |
| SI-T065 無効レジスタ | [12,2,3,4]/9 | E_D("12") | 各々STO16、RCL16 | same/9 | E_D("12"),InvalidRegister(16) | K | V5、固定16個 | P0 |
| SI-T066 ClearStack | [1,2,3,4]/9 | E_E("1"),R0=7 | ClearStack | [0,0,0,0]/0 | I_D、R0=7 | K | V5独自 | P1 |
| SI-T067 ClearRegisters | [1,2,3,4]/9 | E_D("1"),R0=7 | ClearRegisters | [1,2,3,4]/9 | E_D("1"),全Ri=0 | K | V5独自 | P1 |
| SI-T068 整数overflow成功 | [1,127,3,4]/9 | W8、I_D | + | [0x80,3,4,4]/1 | I_E | C0G1 | V5 §7、成功時Drop確認 | P0 |
| SI-T069 RMD保持 | [4,7,3,4]/9 | I_D | RMD | [3,3,4,4]/4 | I_E | K | V5 TEST-068、V5-H p.98/100 | P1 |
| SI-T070 execute_all停止 | [1,2,3,4]/9 | I_D | execute_all([Push(5),Push(0),Divide,Push(7)]) | [0,5,2,3]/9 | I_E,DivisionByZero、Push7未実行 | K | V5逐次契約 | P0 |
| SI-T071 snapshot独立 | [12,2,3,4]/9 | E_E("12"),R0=7 | snapshotを取得しコピーのX/flags/buffer/R0を変更 | [12,2,3,4]/9 | E_E("12"),R0=7 | K | V5公開境界 | P1 |
| SI-T072 初期化reset | [1,2,3,4]/9 | I_E、生成設定W8/HEX/Unsigned、現在設定はDEC/Twos | reset | [0,0,0,0]/0 | I_D、W8/HEX/Unsigned、全Ri=0 | 全0 | V5初期値、HP起動Liftは001 | P1 |
| SI-T073 Float指数編集 | [1e306,2,3,4]/9 | Float、E_D("1e306")/Exp | CHS,BS | [1e-30,2,3,4]/9 | E_D("1e-30")/Exp | K | V5文字編集・有限性、HP未確認 | P1 |
| SI-T074 不正文字集合 | [1,2,3,4]/9 | I_E | 各々InputDigit('-'),InputDigit('１'),InputDigit('G') | same/9 | I_E,各InvalidDigit | K | V5 ASCII入力 | P1 |
| SI-T075 Float幅変更拒否 | [1,2,3,4]/9 | Float、E_D("1") | 各々WS8、SetSignedMode(Unsigned) | same/9 | E_D("1"),NumericModeMismatch | K | V5、実機設定とは差あり | P1 |
| SI-T076 整数CHS最小 | [0x80,2,3,4]/9 | W8 Twos、HEX、E_D("80") | CHS | [0x80,2,3,4]/0x80 | I_E | C1G1 | V5 §7、TBD-007の既存暫定分類 | P1 |

SI-T073は符号切替後に指数を編集しても再Liftしない試験であり、エラーを期待しない。HPの範囲制限を模倣する試験ではない。

## 6. 保留・提案ケース（通常の合格件数へ含めない）

| ID・対象 | 初期スタック／L・入力 | 操作 | 現契約／確認済み部分 | 保留する期待状態・flags | 根拠／解決ゲート | 優先度 |
|---|---|---|---|---|---|---|
| SI-H001 初回幅外桁 | W1 Unsigned HEX、[1,0,0,0]/0、I_E、flags標準 | digit2 | V5はSI-T050。HPの表示X=0は確認 | HPの他段・Lift・編集状態・flagsは未確認、勝手に補わない | H32脚注、CP-01/TBD-013 | P1 |
| SI-H002 DEC符号bit | W8 Twos DEC、[12,2,3,4]/9、E_D("12") | digit8 | V5数学値範囲では拒否。HPには符号bit到達の説明あり | DECキー例として128→raw80を確定するには対象例を確認。Lift/flagsも記録 | H32、CP-01/TBD-013 | P1 |
| SI-H003 負数最終桁削除 | Float、[-1,2,3,4]/9、E_E("-1")、flags標準 | BS | CP-02推奨：[+0,2,3,4]/9、E_E("0")、flags K | **提案期待値**。承認前は製品必須試験にしない。HP動作は別確認 | TBD-021、017 | P1 |
| SI-H004 複数原因 | Float、[1,2,3,4]/9、E_D("1e")/Exp | BASE HEX | 全状態保持のErrは必須 | NumericModeMismatchかIncompleteEntryか。CP-03では前者を推奨 | TBD-022 | P2 |
| SI-H005 HP FLOAT | W64 HEX、[0,0x25E47,3,4]/9、I_D | 実機FLOAT | X=155207.0、Y/Z/T/L=0、W56、Lift Eは過去確認 | 保存領域・f64フラグ対応と新Command未設計。SetNumericModeのテストに流用しない | V5 TEST-072、006 | P1 |
| SI-H006 HP WSIZE領域 | W16、R0=0x1234,R1=0x5678、スタック任意の合法値 | HP WSIZE32、16 | R0がW32で0x56781234、復元幅で元R0/R1 | 直接SetWordSizeには要求しない。全スタック/Lのfixtureは新Command設計時に追加 | V5 TEST-071、004/014 | P1 |
| SI-H007 エラー解除キー | 除算失敗画面、コアはSI-T061の状態 | 実機の数字2 | 表示解除は過去確認 | 数字2を計算にも渡すか、解除だけで消費するか未確認 | TBD-012 | P2 |
| SI-H008 実機最大桁 | W1/3/8/64、全基数、Float仮数/指数 | 上限まで入力後もう1桁 | V5は256文字＋範囲拒否 | 実機の無視／拒否／丸め／表示窓と内部値、Lift、flagsを実測・資料確認 | TBD-013/017/018 | P1 |

保留ケースは期待値が未確定なのでRustの普通の`assert_eq!`ケースへ転記しない。課題管理ID付きの未実装fixtureまたはignore理由を残し、実行済み互換ケースの集計から除外する。

## 7. 網羅・既存仕様との重複整理

| v0.5テスト | 今回の対応 | 扱い |
|---|---|---|
| TEST-010～014 | SI-T001～003、007～009、012～015、018～019 | 既存の低水準スタック試験を保持。今回のphase/Lift/L/flags付きケースで統合試験を拡張 |
| TEST-020～029 | SI-T004～006、010、016、023～026、041～043、061～063 | 既存の状態遷移を具体化。実装で二重登録する必要はなく旧IDを関連タグにする |
| TEST-030/031 | SI-T020～022、065、067 | STO/RCLと全状態失敗を拡張 |
| TEST-032～038 | SI-T027～040、045～052、074 | CHS、編集、符号、上限の拡張。SI-H003は未確定として分離 |
| TEST-039～043 | SI-T044、053～060、075 | 独自設定APIの期待値を具体化。実機変換ケースは分離 |
| TEST-047/048 | SI-T061～065、070 | 単一Commandとexecute_allのロールバック範囲を比較 |
| TEST-061/062 | SI-T030、055～057、064、071～073 | 初期化、有限値、符号0、snapshotを補足 |
| TEST-068/069/070/073 | SI-T069/037/027/011 | 既存確定例を全状態で再利用 |
| TEST-065/071/072 | SI-H001～008 | 未確認を埋めない。HP用APIの将来受入枠 |
| 新規追加が必要 | SI-T002、009、015、019、026、028、035、036、040、041/042対比、043、050、051/052、056、060、062、071、073～076、SI-H001～004 | 繰返し・履歴・複合操作・全領域原子性と仕様穴の検出 |

上表は範囲対応であり、既存試験の全入力例を読み替えるものではない。各新規実装テストに`SI-Txxx`を唯一の主IDとして付け、旧IDは説明コメントに残す。

追加パラメータ展開：

1. SI-01～21の4状態で到達可能な組合せを列挙し、成功後のphaseとLiftが表と一致すること。mode別CHSを重複分類しない。
2. ENTER/CLx/Swap/Roll/RCL/STOをE_D/E_Eでも実行し、Editingの開始時にすでに行ったLiftを繰り返さないこと。
3. WordSize=1,2,3,4,8,16,32,63,64、全基数、全SignedModeで最大rawと1桁超過を検査。DEC数学値上限とraw上限を区別する。1補数の±0はTBD-008に従いHP境界期待値を未確認のまま分離する。
4. 全失敗で内部EntryPartを含む全状態を比較。失敗直後の数字・BS・CLx・同じ演算再試行を追加する。Floatの負0を普通のPartialEqだけで見逃さない。
5. 原子性の検証では予期した値だけでなく、変わってはいけない記憶領域とF0～3も比較する。

## 8. Rustへの転記指針

```rust
// fixture概念。テストモジュール内の型であり公開APIへ追加しない。
struct TransitionCase {
    id: &'static str,
    initial: TestState,     // 全stack/L/Ri/flags/config/input/part
    steps: Vec<Command>,
    expected: TestState,
    expected_error: Option<CalcError>,
    source: &'static str,
}
// TestState / assert_state_bits_equalはテスト支援型・関数。
// Number::Floatはto_bits()で比較、IntegerはWord::bits()で比較。
```

製品実装後、モジュール内のprivate状態試験と`crates/rdp-core/tests/`の公開API試験を分離して`cargo test -p rdp-core`で実行する。現Cargo.tomlには当該crateがないため、今回このコマンドが通ると主張しない。CLI側はキー列と完成数値Push列を分け、コアfixtureの期待Snapshotを使って検証する。
