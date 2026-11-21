# rdp-calc スタック・入力 単体テスト仕様

製品仕様（TBD-017/021、DD-049）：Float編集でBS後に仮数の符号だけが残る場合、bufferを `0`、Xを `+0.0` とし、Editing / MantissaのままLiftを保持する。次の数字は追記する。IA-G012（旧SI-H003）を製品受入試験に含める。

版：SI-1.1 / 2026-10-31 / 基準develop：`eecb806`（旧SI-1.0のfixtureを継承）

対応：[詳細設計](rdp-calc-stack-input-detailed-design.md)、[詳細設計v0.6](rdp-calc-detailed-design.md)。本書は実行結果ではなく、実装時のfixture仕様。SI-T001～076はrdp-inputの結合試験で個別に実行できる。既存TEST番号はv0.5の試験仕様を指す。

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
| SI-T031 点から開始 | [8,2,3,4]/9 | Float、I_E | 点,digit("5") | [0.5,8,2,3]/9 | E_E("0.5") | K | TBD-017 | P1 |
| SI-T032 重複点 | [1.2,2,3,4]/9 | Float、E_D("1.2") | 点 | same/9 | E_D("1.2"),Err | K | V5文法、InputError::InvalidNumber（CP-03確定） | P1 |
| SI-T033 Integer点 | [1,2,3,4]/9 | E_D("1") | 点 | same/9 | E_D("1"),NumericModeMismatch | K | V5 | P1 |
| SI-T034 指数と符号 | [1,2,3,4]/9 | Float、E_D("1") | EEX,digit("2"),CHS | [0.01,2,3,4]/9 | E_D("1e-2")/Exp | K | CE p.58、V5、値はf64参照のto_bits | P1 |
| SI-T035 指数のBS | [0.01,2,3,4]/9 | Float、E_E("1e-2")/Exp | BS,BS,BS | [1,2,3,4]/9 | E_E("1")/Mantissa | K | TBD-017（実機は未確認） | P1 |
| SI-T036 最終桁BS | [1,2,3,4]/9 | E_E("1") | BS,digit("2") | [2,2,3,4]/9 | E_E("02") | K | TBD-017。0をバッファに残す | P1 |
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
| SI-T053 幅縮小／拡大 | [0x1234,0xFF,3,4]/0x1234 | HEX、E_E("1234"),R0=0x5678 | WS8,WS16 | [0x34,0xFF,3,4]/0x34 | I_E,W16,R0=0x78 | K | TBD-004、H32のスタックのみ一致 | P1 |
| SI-T054 符号形式変更 | [0xFF,2,3,4]/9 | W8、HEX、E_D("FF") | SetSignedMode(Unsigned) | [0xFF,2,3,4]/9 | I_D、Unsigned、X数学値255 | K | V5-H p.35/100 | P1 |
| SI-T055 Integer→Float | [5,2,3,4]/9 | HEX、E_D("5"),R0=7 | MODE Float | [5.0,2.0,3.0,4.0]/9.0 | Float、DEC、I_D、R0=7.0 | K | V5独自厳密変換 | P1 |
| SI-T056 変換全体拒否 | [5,2,3,4]/9 | W64 Unsigned、E_E("5"),R0=9007199254740993 | MODE Float | same/9 | Integer、E_E("5"),Calculation(IntegerOutOfRange) | K | V5厳密変換、非表現可能なRi | P0 |
| SI-T057 Float→Integer拒否 | [1.5,2,3,4]/9 | Float、E_D("1.5") | MODE Integer | same/9 | Float、E_D("1.5"),IntegerOutOfRange | K | V5厳密変換、小数切捨て禁止 | P0 |
| SI-T058 Floatで基数 | [5,2,3,4]/9 | Float、E_E("5") | BASE HEX | same/9 | Float、E_E("5"),NumericModeMismatch | K | V5 API差、CE p.59とは別 | P1 |
| SI-T059 Floatで同基数 | [5,2,3,4]/9 | Float、E_D("5") | BASE DEC | [5,2,3,4]/9 | Float、I_D | K | V5 | P1 |
| SI-T060 同mode | [10,2,3,4]/9 | HEX、E_E("A") | MODE Integer | [10,2,3,4]/9 | HEX、I_E | K | V5同mode契約 | P1 |

SI-T056は有限f64への精度欠落、SI-T057は非整数の変換拒否で、ともにCalculation(IntegerOutOfRange)。CP-03で確定し全状態不変も検証する。

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
| SI-T076 整数CHS最小 | [0x80,2,3,4]/9 | W8 Twos、HEX、E_D("80") | CHS | [0x80,2,3,4]/0x80 | I_E | C1G1 | V5 §7、TBD-007のCHS規則（C保持/G更新） | P1 |

SI-T073は符号切替後に指数を編集しても再Liftしない試験であり、エラーを期待しない。HPの範囲制限を模倣する試験ではない。

## 6. 保留・提案ケース（通常の合格件数へ含めない）

| ID・対象 | 初期スタック／L・入力 | 操作 | 現契約／確認済み部分 | 保留する期待状態・flags | 根拠／解決ゲート | 優先度 |
|---|---|---|---|---|---|---|
| SI-H001 初回幅外桁 | W1 Unsigned HEX、[1,0,0,0]/0、I_E、flags標準 | digit2 | V5はSI-T050。HPの表示X=0は確認 | HPの他段・Lift・編集状態・flagsは未確認、勝手に補わない | H32脚注、TBD-013（CLI CP-01とは別） | P1 |
| SI-H002 DEC符号bit | W8 Twos DEC、[12,2,3,4]/9、E_D("12") | digit8 | V5数学値範囲では拒否。HPには符号bit到達の説明あり | DECキー例として128→raw80を確定するには対象例を確認。Lift/flagsも記録 | H32、TBD-013（CLI CP-01とは別） | P1 |
| SI-H003 負数最終桁削除 | Float、[-1,2,3,4]/9、E_E("-1")、flags標準 | BS | [+0,2,3,4]/9、E_E("0")、flags K | 製品確定。実機は未確認 | DD-049、017 | P1 |
| SI-H004 複数原因（移管済み） | Float、E_D("1e") | BASE HEX | CP-03でCalculation(NumericModeMismatch)に確定 | 通常試験IA-G004へ移管、ここでは重複実行しない | TBD-022 Resolved | P0 |
| SI-H005 HP FLOAT | W64 HEX、[0,0x25E47,3,4]/9、I_D | 実機FLOAT | X=155207.0、Y/Z/T/L=0、W56、Lift Eは過去確認 | 製品対象外の実機参照例。SetNumericModeの受入には厳密な全領域変換を用いる | V5 TEST-072、006 | P1 |
| SI-H006 HP WSIZE領域 | W16、R0=0x1234,R1=0x5678、スタック任意の合法値 | HP WSIZE32、16 | R0がW32で0x56781234、復元幅で元R0/R1 | HP式WSIZEの参考例。製品の幅変更はSI-T053で確認 | V5 TEST-071、004/014 | P1 |
| SI-H007 エラー解除キー | 除算失敗画面、コアはSI-T061の状態 | 実機の数字2 | 表示解除は過去確認 | 数字2を計算にも渡すか、解除だけで消費するか未確認 | TBD-012 | P2 |
| SI-H008 実機最大桁 | W1/3/8/64、全基数、Float仮数/指数 | 上限まで入力後もう1桁 | V5は256文字＋範囲拒否 | 実機の無視／拒否／丸め／表示窓と内部値、Lift、flagsを実測・資料確認 | TBD-013/017/018 | P1 |

SI-H003/004の製品期待値はIA-G012/004に移管済み。ほかのSI-HはHP実機の参考例であり、製品の受入件数から除く。

## 7. 網羅・既存仕様との重複整理

| v0.5テスト | 今回の対応 | 扱い |
|---|---|---|
| TEST-010～014 | SI-T001～003、007～009、012～015、018～019 | 既存の低水準スタック試験を保持。今回のphase/Lift/L/flags付きケースで統合試験を拡張 |
| TEST-020～029 | SI-T004～006、010、016、023～026、041～043、061～063 | 既存の状態遷移を具体化。実装で二重登録する必要はなく旧IDを関連タグにする |
| TEST-030/031 | SI-T020～022、065、067 | STO/RCLと全状態失敗を拡張 |
| TEST-032～038 | SI-T027～040、045～052、074 | CHS、編集、符号、上限の拡張。SI-H003の製品期待値はIA-G012へ移管 |
| TEST-039～043 | SI-T044、053～060、075 | 独自設定APIの期待値を具体化。実機変換ケースは分離 |
| TEST-047/048 | SI-T061～065、070 | 単一Commandとexecute_allのロールバック範囲を比較 |
| TEST-061/062 | SI-T030、055～057、064、071～073 | 初期化、有限値、符号0、snapshotを補足 |
| TEST-068/069/070/073 | SI-T069/037/027/011 | 既存確定例を全状態で再利用 |
| TEST-065/071/072 | SI-H001～008 | 製品対象外の実機参照例 |
| 旧版から追加した試験観点 | SI-T002、009、015、019、026、028、035、036、040、041/042対比、043、050、051/052、056、060、062、071、073～076、SI-H001～004 | 繰返し・履歴・複合操作・全領域原子性と仕様穴の検出 |

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
    steps: Vec<InputEvent>,
    expected: TestState,
    expected_error: Option<SessionError>,
    source: &'static str,
}
// TestState / assert_state_bits_equalはテスト支援型・関数。
// Number::Floatはto_bits()で比較、IntegerはWord::bits()で比較。
```

製品実装後、private状態試験は各所有モジュール内、Core公開API試験は`crates/rdp-core/tests/`、編集・協調原子性の公開API試験は`crates/rdp-input/tests/`へ分離し、それぞれ`cargo test -p rdp-core`、`cargo test -p rdp-input`で実行する。CLI側はキー列と完成数値Push列を分け、各所有層のSnapshotで検証する。移設・重複・保留は第9節、実装前提と統合検証の順序は[詳細設計](rdp-calc-detailed-design.md) 第12章に従う。

## 9. IA-1.0移行・追加テスト（現行正本）

以下は実装時の必須仕様であり、テスト実行済みとはしない。旧SI-T001～076のID・数値期待値を維持し、編集fixtureはHpInputSessionへ移設。旧InputDigit/点/EEX/BSはKey、残りはInputEvent::Commandへ変換する。Core fixtureにEntryPhaseを追加しない。InputError5種はSessionError::Input、それ以外はCalculationで比較。SI-T032はInput(InvalidNumber)、SI-T056はCalculation(IntegerOutOfRange)に確定。SI-T024の拒否はCoreではなくセッション。SI-T071はInputSnapshotを変更し独立性を検査する。

| 旧ID群 | 実行層・対応 |
|---|---|
| SI-T001～076 | セッション状態遷移の統合試験として保持。Idleのみの演算ケースも従来fixtureを再利用 |
| TEST-010～019、算術/flags/word試験 | Core単体。文字列を使わずNumberで構築 |
| TEST-020～038 | キー操作はrdp-inputへ。TEST-032のCLI 0xGだけIA-L012へ移す |
| TEST-043/053/054/056/060 | CLI parse/表示/終了コードはIA-L群と対応、Core formatはそのまま |
| TEST-029、SI-T024 | Editing Push拒否をセッションへ配置変更、IA-C002と対比 |
| SI-H003 | 製品試験IA-G012へ移管。製品の符号・削除規則は017 |
| SI-H004 | IA-G004へ移管して確定試験化、旧IDは追跡タグのみ |

追加ケースはIA-L（CLI）、IA-C（Core）、IA-G（共通キー/セッション）。全失敗は全snapshotと内部part、全成功は4段/L/flags/設定/記憶/Liftを比較する。Floatはto_bits、Ones負0はraw比較。明示しないフィールドは保持。L群のparse単体はCoreを呼ばず、副作用がないことをspyで確認する。

### 9.1 CLI数値解析と実行

特記なければW8/DEC/Twos、表のrawはHEX。parseのみはCalculator状態なし。submit試験の共通初期状態はstack=[5,2,3,4],L9,E、flags=F0..3=1010/C1/G1、R0=7。

| ID | 入力・条件 | 期待値・エラー |
|---|---|---|
| IA-L001 | 0x12,0x123,0x1FF,0x100を各独立parse | raw12,23,FF,00 |
| IA-L002 | Unsigned 0,255,256,-1,-256 | raw00,FF,00,FF,00（数学表示0,255,0,255,0） |
| IA-L003 | Twos 127,128,-128,-129 | raw7F,80,80,7F、数学表示127,-128,-128,127 |
| IA-L004 | Ones 127,128,-127,-128 | raw7F,80,80,7F、数学表示127,-127,-127,127 |
| IA-L005 | -123、各SignedMode | Unsigned85/Twos85/Ones84 |
| IA-L006 | -0、各SignedModeとFloat | Unsigned00/Twos00/OnesFF/Float負0bit |
| IA-L007 | W1、各形式、0,1,2,-1 | 非負raw0,1,0。負1はUnsigned/Twos=1、Ones=0 |
| IA-L008 | W64 Unsigned 18446744073709551615,18446744073709551616 | FFFFFFFFFFFFFFFF,0、ホスト整数overflowなし |
| IA-L009 | W64、0x＋Fを254桁、続いて255桁の別試行 | 256文字はFFFFFFFFFFFFFFFF、257文字はLiteral(InputTooLong) |
| IA-L010 | 単独-、全SignedMode/Float | 正0。Ones負0やFloat負0にしない。トークン分類もLiteral |
| IA-L011 | -1,-123、DEC | 通常負数。IA-L005/006と同じ表現式、符号単独規則を誤適用しない |
| IA-L012 | 0xG、0b2、0o8 | Literal(InvalidDigit)、最初の不正文字と接頭辞の基数 |
| IA-L013 | 空、+、0x、--1、0d-1、1_0、全角１、内部空白 | parse単体でInvalidNumber（形不正）またはInvalidDigit（数字列不正）。1_0/全角１はInvalidDigit、他はInvalidNumber |
| IA-L014 | Float 1e-,NaN,Inf,0x1 | InvalidNumber。Float 1e999はFloatingPointOutOfRange。Core未呼出 |
| IA-L015 | 257文字かつ末尾G | InputTooLong優先、桁検査より前 |
| IA-L016 | HEX設定で-0d123、DECで0x123 | raw85,23、表示radixは各元設定を保持 |
| IA-L017 | submit pending=0x123 | stack=[35,5,2,3],L9,E、flags/R0保持、pending消去 |
| IA-L018 | submit pending=0、next=Divide | Calculation(DivisionByZero)、Core全体とpending=0保持 |
| IA-L019 | submit pending=0xG、next=Divide | Literal InvalidDigit、Core呼出なし、全状態/pending保持 |
| IA-L020 | 通常トークン列5 0 / | [0,5,5,2],L9,E、除算失敗、先行2Push保持。初期状態は本節共通 |
| IA-L021 | -、sub、keys内-を分類 | Literal正0、Command Subtract、InputEvent Command Subtract。減算エイリアス衝突なし |
| IA-L022 | Syntax/Literal/Input/Calculation/Io | 終了2/2/1/1/3、stderr診断、失敗Batchは最終Xをstdoutへ出さない |

長さ・形・左端桁・Float変換という順序を固定。Floatで数字列が不完全な1e-はCLI InvalidNumber、HP編集中のIncompleteEntryとは区別する。L013の単独+はparse単体だけの試験でありコマンド識別ではAdd。

追加の境界パラメータ：W=1,2,3,4,8,16,32,63,64、全基数・全SignedModeで0、2^W-1、2^W、各形式の数学的最小/最大、その外側±1を展開する。正値raw=n mod 2^W、負値はIA §2の表現式を独立な多倍長参照計算で求める。これはCore入力の切り捨て試験には流用しない。

### 9.2 Coreの完成値API

初期W8/DEC/Twos、[5,2,3,4],L9,E、flags/R0はL群と同じ。

| ID | 操作・条件 | 期待 |
|---|---|---|
| IA-C001 | 幅16で作ったWord raw123をPush | IntegerOutOfRange、全状態不変（自動マスク禁止） |
| IA-C002 | Push(3)、編集型を持たないCore | [3,5,2,3],L9,E。InputInProgressを返す経路なし |
| IA-C003 | Enter→Push(3) | [3,5,2,3],L9,E、Enter自身のlift後は上書き |
| IA-C004 | Start(1)→ReplaceX(12) | [12,5,2,3],L9,E、liftは一度 |
| IA-C005 | Enter→Start(1)→ReplaceX(12) | [12,5,2,3],L9,D、編集ではLiftを有効化しない |
| IA-C006 | IntegerでPush(Float(NaN)) | FloatingPointOutOfRangeがNumericModeMismatchに優先、不変 |
| IA-C007 | IntegerでPush(Float(1.0)) | NumericModeMismatch、不変 |
| IA-C008 | 内部fixtureのflags上位bit破損＋X0でDivide | InvariantViolation優先。演算せず破損を含め状態不変 |
| IA-C009 | FloatでEditBit index255 | InvalidBitIndex優先、mode不一致は次順位、無効bitシフトしない |
| IA-C010 | W64 Unsigned R0=9007199254740993→Float | IntegerOutOfRange、全領域一括取消、SI-T056と対応 |
| IA-C011 | X0,Y5でDivide/Remainder各独立 | DivisionByZero、stack/L/flags/設定/Ri不変 |
| IA-C012 | snapshotコピーのX/R0/flags変更 | 実Core不変、編集fieldなし |
| IA-C013 | execute_all Push5,Push0,Divide,Push7 | [0,5,5,2],L9,E、先行成功維持、Push7なし |
| IA-C014 | 順位選択関数に同時候補を与える内部単体試験 | IA §4の全順位・同列順を総当たり。対象順X/Y/Z/T/L/Riも固定 |
| IA-C015 | 不正mode＋値依存条件のある命令 | mode適合前に演算しない。FloatのAndはNumericModeMismatch、結果範囲を評価しない |
| IA-C016 | 候補検証不整合を内部fault injection | InvariantViolation、実state不変。公開APIに注入機能を作らない |

InvalidWordSize/InvalidRegisterCount/AllocationFailedは生成・確保経路、その他順位候補は命令経路で単一原因も検査する。内部順位関数の総当たりだけで到達可能な競合試験を代替しない。OSのOOM abortは試験対象外。

### 9.3 GUI相当・協調原子性

| ID | 操作・条件 | 期待 |
|---|---|---|
| IA-G001 | 共通状態からDigit1,Digit2 | [12,5,2,3],L9,E、Editing("12")、part=Mantissa |
| IA-G002 | Float共通状態、Digit1,Digit2,CHS,BS | [-1,5,2,3],L9,E、Editing("-1")、flags保持 |
| IA-G003 | W8 HEX Editing("FF")→Digit F | Input経由Calculation(IntegerOutOfRange)、全セッション保持。CLIならFFF→FFを受理 |
| IA-G004 | Float Editing("1e")、SetRadix HEX | Calculation(NumericModeMismatch)、全状態不変。旧SI-H004を確定移管 |
| IA-G005 | Float Editing("1e")、Divide | Input(IncompleteEntry)、previewでゼロ判定しない、全保持 |
| IA-G006 | Editing("12")、Push3 | Input(InputInProgress)、全保持。直接CoreのIA-C002と対比 |
| IA-G007 | Float Editing("1.2")、点 | Input(InvalidNumber)、全保持。SI-T032のvariant確定 |
| IA-G008 | Editing("0")で除算失敗→Digit2 | SI-T062と同じEditing("02")、失敗後も編集継続 |
| IA-G009 | Enter→Digit3 | SI-T004の値、Disabled保持。CLI完成PushのIA-C003とLiftを区別 |
| IA-G010 | InputSnapshotのbuffer/part/Core snapshotを変更 | 実セッション全状態不変。SI-T071を拡張 |
| IA-G011 | HP Editing中にCLI pending入力への切替要求 | InputInProgress、Core・HP編集・pending不変、二重編集禁止 |
| IA-G012 | Float Editing("-1")でBS | Editing("0")、X=+0.0、Mantissa、Lift保持。次の2でbuffer="02" |

IA-G003の幅外エラーはSessionError::Calculation(IntegerOutOfRange)。IA-G012は製品受入に含める。SI-H001/002/005～008は実機比較用。

### 設定変更の製品受入（TBD-001/006/008）

- リセット直後のLiftがDisabledであることと、次の初回入力がY/Z/Tを保持することを確認する。
- 編集中の `oneszero raw` は旧設定でfinishしてから設定を変更する。Idleで切り替えた場合は全値・flags・Liftが等しいことを確認する。リセット後は生成時の設定に戻る。
- Integer→FloatでXが変換可能でも、R0が `2^53+1` なら全状態を保持して失敗する。対応する成功例はW64で `2^53`。Float→Integerでは小数部のある `1.5` を拒否する。
