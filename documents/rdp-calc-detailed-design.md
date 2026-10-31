# rdp-calc Rust詳細設計書

- 文書版：v0.6 CP-01～03・入力API分離（2026-10-31）
- 作成日：2026-10-19 / 改訂日：2026-10-31（日本時間）
- 対象：Rust Stable / Cargo Workspace / Windows・VSCode
- 状態：実装用インターフェースと暫定動作を定義。HP-16C完全互換の認証済み仕様ではない。
- 文字コード：UTF-8、改行：LF
- 関連文書：[未確定事項一覧](rdp-calc-tbd-list.md)、[設計判断記録](rdp-calc-design-decisions.md)

入力API・CLI変換・エラー順位の正本は[入力API設計 IA-1.0](rdp-calc-input-api-design.md)。CP-01～03を確定反映した。キー遷移は[SI-1.1](rdp-calc-stack-input-detailed-design.md)、試験の移設・追加は[試験仕様](rdp-calc-stack-input-test-cases.md)を参照する。算術とHP互換の未解決境界は維持する。

## 第1章：概要

### 1.1 目的・対象・範囲

rdp-calcはHP-16C相当の整数演算と4段RPNスタックを備える電卓である。計算エンジンを独立したRustライブラリにし、対話CLI、ワンショットCLI、将来のGUIから共通のCommand APIで利用する。本書は構造、関数契約、入力とスタックの状態遷移、整数演算、エラー、CLI、テスト、実装順序を定義する。関数本体とGUI実装は本設計作業の対象外。HP-16Cのプログラミング機能は製品のサポート対象外とし、記録・保存・実行、分岐・ラベル・サブルーチン・PC・戻りスタックを実装しない。自動化・条件分岐・繰返しは外部スクリプトからCLIを呼び出して行う。

整数は1～64ビット、Unsigned / OnesComplement / TwosComplementに対応する。浮動小数点はf64を採用し、実機の十進演算・表示桁・丸めとの完全一致を要求しない。実機のキーコード、表示窓、物理メモリ容量の再現は必須としないが、演算結果・スタック・フラグの互換性は優先する。直接設定API、CLIの一括数値入力、FloatのRMDなどは独自拡張として区別する。ABSはHP-16Cにも存在する標準機能である（SRC-08印刷p.44）。

### 1.2 仕様の区分

| 表示 | 意味 | 実装・試験の扱い |
|---|---|---|
| 要件 | 今回の依頼で指定された事項 | 必須 |
| 設計確定 | 本書で採用するrdp-calcの構造・契約 | 実装基準。実機確認済みという意味ではない |
| 継承 | v0.1～v0.3から採用した仕様 | 出典は先行設計。外部資料での再検証と区別 |
| 外部確認 | 今回、マニュアル本文を読んで確認した限定的事実 | 出典・確認範囲を明示 |
| 暫定 / TBD-nnn | 動く実装のための仮仕様、または未定義 | 暫定試験と互換性試験を分離。互換性合格を宣言しない |

「設計確定」と「HP-16C互換性の確定」は別軸である。例えば原子的なエラー処理は設計確定だが、実機のエラー復帰との一致はTBD-012。表中の「暫定」はその行全体に適用し、外部確認済みの部分がある場合だけ別記する。

### 1.3 一次資料と確認範囲

| ID | 資料 | 使用範囲 |
|---|---|---|
| SRC-01 | 添付 `rust-rpn-v0.1.md` | 全文。構成、責務、機能・フェーズ |
| SRC-02 | 添付 `rust-rpn-v0.2.md` | 全文。型、API、初期値、エラー、CLI |
| SRC-03 | 添付 `rust-rpn-v0.3.md` | 全文。入力状態、命令表、暫定規則 |
| SRC-04 | [先行チャット](https://chatgpt.com/c/6ad49338-fe6c-83ec-acd1-e06defb64aeb) | URL本文は直接取得不可。添付3資料を正本として利用 |
| SRC-05 | [HP-16C Owner's Handbook書誌](https://literature.hpcalc.org/items/589) | Hewlett-Packard、Revision D、April 1984、00016-90001。PDF全文は取得サイズ制限により今回未照合 |
| SRC-06 | [HP作成Handbook転載、表示ページ38／印刷ページ36](https://www.manualslib.com/manual/4100736/Hp-Hp-16c.html?page=38) | Flags節の本文。F0～2、F3、F4、F5の役割 |
| SRC-07 | [同、表示ページ45／印刷ページ43](https://www.manualslib.com/manual/4100736/Hp-Hp-16c.html?page=45) | SUBのBorrow、DIVの余りとCarry、RMDの符号、除算範囲外条件 |

SRC-06/07はメーカー著作の一次資料の転載であり、OCRの数式欠落がある。読めた文章の内容だけを外部確認とする。SRC-03が引用するAppendix A/Bの完全な原表は今回未照合。先行文書の「確認済み」という表記は出典経路として維持するが、本書で未検証の細部まで外部確認済みへ昇格させない。Collector's Editionは検索と相互確認に使い、確定事項は原版の該当画像へ戻って照合する。CE固有の保守・セルフテスト仕様は採用しない。

外部確認した限定事項：F3は先頭ゼロ表示を制御し、DECと浮動小数点では先頭ゼロが抑制される。F4はCarry、F5はOut-of-range。DIVのCarryは余りが非ゼロならSet、ゼロならClear。RMDの非ゼロ結果の符号は被除数Yに従う。SUBではBorrowと加算のCarryを混同しない。これ以外の命令ごとの更新規則は第7章で区分する。

### 1.3.1 v0.5の確認資料

| ID | 資料 | 確認範囲 |
|---|---|---|
| SRC-08 | 添付 `hp16c.pdf` 原版画像 | 印刷p.25、32、35、44、47、49、57、67、97～100を目視確認。PDFページは印刷ページ+2。書誌の改訂記号は今回確認していない |
| SRC-09 | 添付 `HP-16c-Collectors-Edition-Owners-Handbook.pdf`、Edition 2, November 2025 | 本文抽出による検索、原版との該当節比較。CEの印刷p.25、32、35、44、47、49、57、67、97～100の共通内容を使用。確認したCEの本文ではPDFページ=印刷ページ+4 |
| SRC-10 | ユーザーの2026-10-25設計方針 | プログラミング機能を対象外とし、CLIを外部スクリプトから利用 |

照合結果・変更理由・残る判断は[マニュアル照合記録](rdp-calc-manual-review.md)に記す。以下の「外部確認」は文書上の仕様確認であり、Rustの実装試験・実機試験の合格を意味しない。

### 1.4 機能範囲と仕様番号

| 仕様ID | 内容 | 今回の到達点 |
|---|---|---|
| SPEC-ARCH | UI非依存・非循環の構成 | 設計確定 |
| SPEC-DATA | 型と不変条件 | 設計確定 |
| SPEC-TXN | 1コマンドの原子性 | 設計確定 |
| SPEC-STACK | 固定4段、LAST X、命令効果 | 基本操作は継承、命令細部は暫定 |
| SPEC-INPUT | 入力状態・Push・キー入力 | 拡張契約は設計確定、実機との一致はTBD |
| SPEC-INT | 整数演算とフラグ | 実装可能な暫定規則を定義 |
| SPEC-FLOAT | f64、有限値、変換 | f64は要件、切替は暫定 |
| SPEC-ERROR | エラー分類と状態保持 | 設計確定、実機番号はTBD |
| SPEC-CLI | 構文と実行・終了コード | 設計確定 |
| SPEC-PROGRAM | HP内蔵プログラム機能 | 対象外として確定。自動化はCLI経由の外部スクリプト |
| SPEC-ADV | LJ、倍長、間接指定など | 未対応を明示、無断削除しない |

## 第2章：ソフトウェアアーキテクチャ

### 2.1 Workspace

計画構成は`crates/rdp-core`、`crates/rdp-input`、`crates/rdp-cli`。Coreのinputをrdp-inputへ移し、CLI parserはCLI内に置く。文書は`documents/`、共有試験データは`tests/scenarios/`。現Cargoは単一crate・edition 2024の雛形であり、workspace化とedition選択は実装時に行う。

### 2.2 依存関係と公開境界

CLI→Core、CLI/GUI→rdp-input→Core。CoreからCLI/rdp-input/端末への依存は禁止。[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §1・3が構成図と公開型の正本。Core内は旧DAGからinputノードとそれへの依存のみ除去し、数値・演算部品の依存は維持する。configは独立、error→config、number→word/config/error、state→stack/register/flags/word/config/number/error、calculator→state/command/演算部品。StackLiftはCore、EntryPhase/EntryPart/InputStateはrdp-input。libで公開型を再公開し内部state/stack/registerはpub(crate)。Snapshotは独立コピー、restoreは提供しない。

## 第3章：データ構造設計

### 3.1 共通規約

フィールド表の`private`は定義モジュール内部、`crate`はpub(crate)、`public`はpub。enumの公開variantは型と同じ公開範囲。初期値「引数」はコンストラクターで指定し、暗黙のDefaultを設けない。Number、Word、WordSize、WordConfig、設定enum、Flags、FlagAction、FlagUpdate、EvalResultはCopy可。CalculatorStateとSnapshotはClone。NumberにEqを要求しない。原子性検査で浮動小数点の符号付きゼロまで比較する場合はto_bitsを使う。

### 3.2 所有構造・設定・スナップショット

| 型（公開範囲）・役割 | フィールド：型／可視性 | 初期値・不変条件 |
|---|---|---|
| Calculator（public）：唯一の状態変更窓口 | state: CalculatorState / private; initial_config: CalculatorConfig / private | with_configの検証済み設定。resetは生成時設定へ戻す |
| CalculatorState（crate）：現在の状態 | stack: RpnStack; stack_lift: StackLift; registers: RegisterBank; flags: Flags; word_config: WordConfig; radix: Radix; numeric_mode: NumericMode / 全crate | 全Numberのvariantがnumeric_modeに一致。整数は現在幅に収まる。浮動小数点は有限 |
| CalculatorConfig（public）：生成設定 | word_config: WordConfig; radix: Radix; numeric_mode: NumericMode; register_count: usize / 全public | 16bit、TwosComplement、Hexadecimal、Integer、16。個数1～1024を受理。16個は独自暫定TBD-011 |
| CalculatorSnapshot（public）：値の複製 | stack: [Number; 4]; last_x: Number; registers: Vec&lt;Number&gt;; flags: Flags; radix: Radix; word_config: WordConfig; numeric_mode: NumericMode; stack_lift: StackLift / 全public | snapshot時点のコピー。全レジスタを含む。編集はInputSnapshotへ分離 |
| RpnStack（crate）：4段とLAST X | values: [Number; 4]; last_x: Number / 全private | モードに合う0、順序X/Y/Z/T。空状態なし |
| RegisterBank（crate）：記憶領域 | values: Vec&lt;Number&gt; / private | 指定個数のモード対応0。運用中の長さ不変 |

浮動小数点で生成するとradixはDecimalのみ受理し、初期WordConfigは将来整数へ戻すため保持する。CLIの`--float`は明示基数指定がなければDecimalを設定する。生成後に補数・幅・モードを変更する規則は第4章とTBD-004～006。

### 3.3 数値・設定・入力・フラグ

| 型（公開範囲）・役割 | フィールドまたはvariant／可視性 | 初期値・不変条件 |
|---|---|---|
| Number（public）：値 | Integer(Word), Float(f64) / public | 状態内ではモード一致・有限。外部でFloat(NaN)を作れてもexecuteで拒否 |
| Word（public）：生ビット | bits: u64 / private | from_bitsで指定幅へマスク。幅を自身に保持しないため、状態への取込み時にも検査 |
| WordSize（public）：幅 | 0: u8 / private | new(1..=64)だけが生成可能 |
| WordConfig（public）：整数解釈 | size: WordSize; signed_mode: SignedMode / public | 16、TwosComplement |
| SignedMode（public） | Unsigned, OnesComplement, TwosComplement | TwosComplement |
| Radix（public） | Binary, Octal, Decimal, Hexadecimal | Hexadecimal、基数2/8/10/16 |
| NumericMode（public） | Integer, FloatingPoint | Integer |
| InputState（rdp-input private） | phase: EntryPhase; buffer: String; part: EntryPart / private | Idle、空、Mantissa。LiftはCoreに分離 |
| EntryPhase（rdp-input public） | Idle, Editing | Idle。モードや指数部とは独立 |
| StackLift（public） | Enabled, Disabled | Disabled（独自暫定、TBD-001） |
| EntryPart（rdp-input public） | Mantissa, Exponent | Mantissa。整数では常にMantissa |
| Flags（public、変更はcrateのみ） | bits: u8 / private | 0。上位2bit常時0。F0～F5のみ |
| FlagAction（crate） | Keep, Clear, Set | Keep |
| FlagUpdate（crate） | carry: FlagAction; out_of_range: FlagAction / crate | 両方Keep |
| EvalResult（crate） | value: Number; flags: FlagUpdate / crate | 演算結果。入力引数から生成 |
| BitwiseResult（crate） | value: Word; flags: FlagUpdate / crate | 現在幅内の結果 |

Wordからi128への変換は数学値の取得用であり、1の補数の負のゼロはその変換で失われる。raw bitsを正本として保持する。Word::is_negativeは符号ビットを見るため1の補数の負のゼロでもtrue、数値比較にはto_signedを使う。

### 3.4 Commandと補助enum

次のvariant集合を今回のAPI設計とする。単項回転とスタック上の回数指定回転を区別する。Mask/BitEditの整数引数はrdp-calc拡張、HPキー互換のX/Y配置はTBD-010。

```rust
pub enum Command {
    Push(Number), Enter, SwapXY, RollUp, RollDown, RecallLastX,
    Add, Subtract, Multiply, Divide, Remainder, ChangeSign, Absolute, Sqrt,
    And, Or, Xor, Not,
    ShiftLeft, ShiftRight, ShiftRightArithmetic,
    RotateLeft, RotateRight, RotateLeftCarry, RotateRightCarry,
    RotateN(RotateKind),
    MaskLeft(u8), MaskRight(u8), EditBit { index: u8, edit: BitEdit },
    CountBits,
    SetRadix(Radix), SetWordSize(WordSize), SetSignedMode(SignedMode),
    SetNumericMode(NumericMode), Store(usize), Recall(usize),
    SetFlag { index: u8, value: bool },
    ClearX, ClearStack, ClearRegisters,
}
pub enum RotateKind { Left, Right, LeftCarry, RightCarry }
pub enum BitEdit { Set, Clear }
```

Commandは操作ごとに明示生成しDefaultを持たない。任意のNumber・indexを受け取れるためexecuteで検証する。TestBitは値を返す独立問い合わせAPI、実機のB?による条件スキップは対象外。boolによる独立照会は外部処理に利用できる。DBL/LJ/実機WSIZE/間接レジスタ命令はこのenumに偽の実装を追加せず、CLIで既知の未対応命令として検出する。

```rust
pub(crate) enum ArithmeticOp { Add, Subtract, Multiply, Divide, Remainder }
pub(crate) enum UnaryOp { ChangeSign, Absolute, Sqrt }
pub(crate) enum LogicOp { And, Or, Xor, Not }
pub(crate) enum ShiftOp { Left, RightLogical, RightArithmetic }
pub(crate) enum LiftEffect { Enable, Disable, Neutral }
pub(crate) enum LastXEffect { Keep, SaveX, Clear }
pub(crate) struct CommandMetadata {
    pub(crate) lift_effect: LiftEffect,
    pub(crate) last_x_effect: LastXEffect,
}
```

CommandMetadataはcalculator内のcrate型。metadata(command, numeric_mode)でLift/履歴効果を分類する。編集終了とFloat Editing CHSはrdp-inputへ移す。Coreは確定値だけを扱いPush/Enter/ClearXは専用経路とする。

### 3.5 CalcError

[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §4の優先順位表が全variantの正本。InvalidDigit/InvalidNumber/IncompleteEntry/InputInProgress/InputTooLongはInputErrorへ移設し、InvariantViolationを追加する。CoreはCLIのLiteralErrorを知らない。全エラーはDebug/Clone/PartialEq/Eq、Display、std::error::Errorを実装しvariant/payloadでテストする。

### 3.6 プログラミング機能の対象外化

ProgramEngine、Instruction、ProgramStatus、ProgramStepの型候補を削除する。プログラムメモリ、ラベル、分岐、Call/Return、PC、戻りスタック、命令実行予算をコアに追加しない。TEST-064はCLIによる外部自動化の試験へ変更する。

### 3.7 概念クラス図

[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §3にCoreと入力アダプターの所有図、CalculatorSnapshot/InputSnapshotの全フィールドを定義する。

## 第4章：モジュール詳細設計

### 4.1 共通関数契約

以下の宣言は関数シグネチャ仕様であり、ビルド可能な実装ではない。`Self`は直前の型、標準型はRust標準ライブラリの型。特記なければ、純粋関数は引数を変更せず、失敗時はResult::Err、crateのmut関数も検証完了前に変更しない。全関数はグローバル状態を参照しない。利用者入力でunwrap、未検査インデックス、幅以上のシフトを実行しない。メモリ枯渇などプロセス資源の完全な回復までは保証しない。

### 4.2 calculator.rs

公開APIとprivate apply_command/apply_value/validate_valueは[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §3。execute/update_valueは候補stateに適用し検証後置換、execute_allは逐次成功保持。Coreのfinish_entryを廃止しrdp-input::finishへ移す。new/reset/with_configの値・flags・生成時設定復元は従来どおり。test_bitの編集ガードはセッションにのみ存在する。

### 4.3 command.rs / config.rs

責務は意味のあるコマンドと軽量設定enumの定義。第3章の型をそのまま使用する。Commandに可変参照・UIコールバック・文字列パーサーを持たせない。

```rust
impl Radix { pub fn base(self) -> u32; }
```

baseはBinary→2、Octal→8、Decimal→10、Hexadecimal→16。無状態・エラーなし。Commandに実行メソッドを付けずCalculatorへ集約する。対応していない命令は第9章の既知未対応一覧に残す。単体試験：TEST-032、053、060。

### 4.4 state.rs

CalculatorState、CalculatorConfig、CalculatorSnapshotを定義。状態検証と初期化を担当し、命令の意味は持たない。

```rust
impl Default for CalculatorConfig { fn default() -> Self; }
impl CalculatorConfig { pub fn validate(&self) -> Result<(), CalcError>; }
impl CalculatorState {
    pub(crate) fn new(config: &CalculatorConfig) -> Result<Self, CalcError>;
    pub(crate) fn validate(&self) -> Result<(), CalcError>;
    pub(crate) fn snapshot(&self) -> CalculatorSnapshot;
    pub(crate) fn reconfigure_word(&mut self, next: WordConfig) -> Result<(), CalcError>;
    pub(crate) fn convert_mode(&mut self, next: NumericMode) -> Result<(), CalcError>;
}
```

validate(config)はレジスタ数とfloat/Decimal整合、validate(state)は全Numberのmode・幅・有限性、flags、固定レジスタ長を確認する。編集状態はHpInputSessionが別に検証する。newは検証後に初期化しAllocationFailedを返せる。snapshotはCalculatorから委譲されるだけで外部へstateを漏らさない。

reconfigure_wordは入力確定済みIntegerのみ。暫定TBD-004：全スタック、LAST X、全記憶レジスタを同時変換し、縮小は低位切詰め、拡大は符号形式によらずゼロ拡張。スタックの縮小/拡大はSRC-08印刷p.32確認済み。LAST Xと記憶領域への同一変換は独自暫定であり、HPでは記憶領域のraw storageを保存して境界を再解釈する（p.67）。RegisterBankの再設計はTBD-004に残し、現方式で記憶領域互換を主張しない。F4/F5はKeep、LiftはNeutral。符号変更だけなら全raw bitsを保持して再解釈（TBD-005）。float中のSetWordSize/SetSignedModeはNumericModeMismatch。SetRadixもFloatでDecimal以外ならNumericModeMismatchとし、旧入力を確定した候補状態ごと破棄する。変更失敗時は候補破棄。

convert_modeの暫定TBD-006：全5スタック値＋全記憶領域を数値変換。Integer→Floatは表現がf64で厳密に保持できる整数のみ受理、負のゼロは-0.0。Float→Integerは有限かつ整数、範囲内かつ往復変換で値が一致するもののみ受理。丸め・切捨て変換はしない。負のゼロはOnesComplementなら負のゼロ、他形式なら0。LAST Xも変換しクリアしない（この厳密変換は独自APIの暫定契約。HPのFLOATはX=Y×2^X、Y/Z/T/LAST Xクリア、幅56、Lift有効でありp.57で確認済み。HP変換命令を追加するかはTBD-006に残す）。radixは双方の切替でDecimal、flags/Lift維持。複数値の一部だけ変換しない。同一モード指定も入力確定を行うが値・基数は変更しない。単体試験：TEST-039～043、061。

### 4.5 rdp-inputの編集器

Core内input.rsを廃止し[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §3のInputState/HpInputSessionへ移設する。編集はstackを直接操作せずStart/ReplaceXとCommandを使う。逐次キーの文法・厳密範囲・256文字上限はSI設計の暫定契約を維持。CLI parse_literalは別実装とする。

### 4.6 stack.rs

RpnStackの低水準操作。数値モード・命令・flagsには依存しない。

```rust
impl RpnStack {
    pub(crate) fn new(zero: Number) -> Self;
    pub(crate) fn values(&self) -> [Number; 4];
    pub(crate) fn x(&self) -> Number;
    pub(crate) fn y(&self) -> Number;
    pub(crate) fn z(&self) -> Number;
    pub(crate) fn t(&self) -> Number;
    pub(crate) fn last_x(&self) -> Number;
    pub(crate) fn lift(&mut self);
    pub(crate) fn drop_stack(&mut self);
    pub(crate) fn swap_xy(&mut self);
    pub(crate) fn roll_up(&mut self);
    pub(crate) fn roll_down(&mut self);
    pub(crate) fn set_x(&mut self, value: Number);
    pub(crate) fn save_last_x(&mut self);
    pub(crate) fn replace_all(&mut self, values: [Number; 4], last_x: Number);
}
```

newは全5値をzeroにする。read系はCopyを返す。lift/drop_stack/循環は第5章の式、set_xはXだけ、save_last_xは現在Xを保存、replace_allは変換済みの5値を交換。型整合は呼出側が検証済み。いずれもエラー・underflowなし。`drop`はRustのデストラクターと紛らわしいためdrop_stackへ改名。単体試験：TEST-010～014。

### 4.7 number.rs / word.rs

数値表現・数学値への変換・安全な幅マスクを担当。演算によるフラグ変更はしない。

```rust
impl Number {
    pub fn as_integer(&self) -> Option<Word>;
    pub fn as_float(&self) -> Option<f64>;
    pub(crate) fn validate(&self, mode: NumericMode, config: &WordConfig)
        -> Result<(), CalcError>;
}
impl WordSize {
    pub fn new(bits: u8) -> Result<Self, CalcError>;
    pub fn bits(self) -> u8;
    pub fn mask(self) -> u64;
}
impl Word {
    pub fn from_bits(bits: u64, size: WordSize) -> Self;
    pub fn bits(self) -> u64;
    pub fn masked(self, size: WordSize) -> Self;
    pub fn is_negative(self, config: &WordConfig) -> bool;
    pub fn is_negative_zero(self, config: &WordConfig) -> bool;
    pub fn to_signed(self, config: &WordConfig) -> i128;
    pub(crate) fn from_integer(value: i128, config: &WordConfig) -> Result<Self, CalcError>;
}
```

as系は異なるvariantならNoneであり暗黙変換しない。WordSize::newのみ幅の検証、maskは64でu64::MAX、他は2^W−1。from_bits/maskedは明示的な情報切捨てAPIでエラーにしないが、Pushは異なる幅で作ったWordの上位ビットを無断で切り捨てずIntegerOutOfRangeとする。

to_signedはUnsignedも0～2^64−1をi128で表現できる。OnesComplementは符号bitがあれば`-((~bits)&mask)`、TwosComplementは符号bitがあれば`bits−2^W`。全メソッドは指定幅にマスクして解釈する。from_integerは表現範囲を検査し範囲外ならErr、0は正のゼロ。負のゼロ生成にはfrom_bitsを使用する。単体試験：TEST-001～009。

### 4.8 arithmetic.rs

ArithmeticOp、UnaryOp、EvalResultを定義。全関数純粋。評価関数は型/幅/有限性を検証し、第7章の結果とFlagUpdateを返す。

```rust
pub(crate) fn eval_binary(op: ArithmeticOp, lhs: Number, rhs: Number,
    config: &WordConfig) -> Result<EvalResult, CalcError>;
pub(crate) fn eval_unary(op: UnaryOp, value: Number,
    config: &WordConfig) -> Result<EvalResult, CalcError>;
fn eval_integer(op: ArithmeticOp, lhs: Word, rhs: Word,
    config: &WordConfig) -> Result<EvalResult, CalcError>;
fn eval_float(op: ArithmeticOp, lhs: f64, rhs: f64) -> Result<EvalResult, CalcError>;
fn integer_sqrt(value: u64) -> u64;
```

二項はlhs=Y、rhs=X。混在型はNumericModeMismatch。ゼロ除算、負数sqrt、非有限結果をErr。内部整数sqrtは0～u64::MAXを正確に切捨て平方根へ変換し、浮動小数点経由にしない。通常の整数範囲外は成功＋GでありErrではない。単体試験：TEST-015～019、044～046、057～059、062。

### 4.9 bitwise.rs

LogicOp、ShiftOp、BitwiseResultを定義。RotateKindとBitEditはcommandから借用するとDAGが不要に上位へ向かうため、これら2つのenumの定義所有元はbitwiseとし、lib.rsからpublic再公開する。commandはbitwiseの型定義にのみ依存する。bitwiseからcommandへの依存は禁止する。

```rust
pub(crate) fn logic(op: LogicOp, lhs: Word, rhs: Option<Word>, size: WordSize)
    -> Result<BitwiseResult, CalcError>;
pub(crate) fn shift(op: ShiftOp, value: Word, config: &WordConfig)
    -> Result<BitwiseResult, CalcError>;
pub(crate) fn rotate(kind: RotateKind, value: Word, count: u8,
    size: WordSize, carry_in: bool) -> Result<BitwiseResult, CalcError>;
pub(crate) fn mask_left(count: u8, size: WordSize) -> Result<BitwiseResult, CalcError>;
pub(crate) fn mask_right(count: u8, size: WordSize) -> Result<BitwiseResult, CalcError>;
pub(crate) fn edit_bit(value: Word, index: u8, edit: BitEdit, size: WordSize)
    -> Result<BitwiseResult, CalcError>;
pub(crate) fn test_bit(value: Word, index: u8, size: WordSize) -> Result<bool, CalcError>;
pub(crate) fn count_bits(value: Word, size: WordSize) -> BitwiseResult;
```

logicはNotならrhs=None、二項ならSome必須、不一致はInvalidConfiguration。rotateのcountは通常0～W、Carry経由0～W+1（最大65）、bit indexは0～W−1、mask countは0～W。shiftは常に1回、rotateはcount回で0回時flags Keep。非キャリー回転はW、キャリー回転はW+1の輪を用いる。入力幅外bitsはIntegerOutOfRange、出力は幅内。count_bitsの個数最大WはそのWビットに必ず収まる。各純粋関数の詳細は第7章、単体試験：TEST-049～052、063。

### 4.10 flags.rs

Flags、FlagAction、FlagUpdateを定義。演算意味を解釈せず指示を適用する。

```rust
impl Flags {
    pub fn get(&self, index: u8) -> Result<bool, CalcError>;
    pub fn carry(&self) -> bool;
    pub fn out_of_range(&self) -> bool;
    pub(crate) fn set(&mut self, index: u8, value: bool) -> Result<(), CalcError>;
    pub(crate) fn apply(&mut self, update: FlagUpdate);
}
impl FlagUpdate {
    pub(crate) const fn keep() -> Self;
    pub(crate) const fn from_conditions(carry: Option<bool>, out_of_range: Option<bool>) -> Self;
}
```

Defaultはbits0。get/setはindex0～5だけ受理、他はInvalidFlagで不変。applyはF4/F5だけ変更しF0～3を保つ。from_conditionsのNoneはKeep、Some(false)はClear、Some(true)はSet。条件付き更新は評価時にClear/Setへ解決され、FlagActionに条件式を格納しない。単体試験：TEST-016～019、044、055。

### 4.11 register.rs

RegisterBankを定義。直接インデックス方式だけを実装する。

```rust
impl RegisterBank {
    pub(crate) fn new(count: usize, zero: Number) -> Result<Self, CalcError>;
    pub(crate) fn get(&self, index: usize) -> Result<Number, CalcError>;
    pub(crate) fn set(&mut self, index: usize, value: Number) -> Result<(), CalcError>;
    pub(crate) fn values(&self) -> &[Number];
    pub(crate) fn replace_all(&mut self, values: Vec<Number>) -> Result<(), CalcError>;
    pub(crate) fn clear(&mut self, zero: Number);
}
```

newは1～1024か検査、try_reserveで確保失敗を返す。get/setは0≤index<lenを検査。replace_allは同一長だけ受理、全値は呼出側で検証済み。clearは個数を保ったまま0へ。get/valuesは不変、他は成功時変更。STO/RCLのLiftはCalculator側。単体試験：TEST-030、031、061。

### 4.12 format.rs

Coreはformat_number(value, radix, mode, config, flags) -> Result<String, CalcError>を維持。F3・基数・符号解釈の表示は変更しない。parse_number/preview_numberをCoreから除去しCLIとキー編集器へ分離する。CLI切り捨て・負数・負0・単独-・接頭辞・上限は[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §2。

format_numberはBIN/OCT/HEXでraw bits、DECでSignedMode解釈、OnesComplementの負0は`-0`。F3=0で先頭ゼロ抑制、F3=1で非DEC整数をceil(W/log2(base))桁へ0詰め（暫定TBD-018）。Floatは有限f64の最短往復可能表現、-0.0は`-0`。幅・接尾文字はCLI、型・mode不一致はErr。既存TEST-043を維持する。

### 4.13 error.rs

第3章のCalcErrorのみを定義。Displayは安定した短い英文を返す契約とし、文字列一致はテスト合格条件にせずvariantとpayloadを検査する。std::error::Error::sourceはNone。失敗時にログ出力、終了、状態変更しない。単体試験：TEST-047、048、054。

### 4.14 外部スクリプトによる自動化

内蔵プログラムAPIは提供しない。外部スクリプトは第9章のワンショットCLIを利用する。同一呼出し内では命令列の状態を継続し、別プロセス間では状態を共有しない。必要な初期設定・数値は毎回明示する。結果はstdout、診断はstderr、終了コードは0/1/2/3。スクリプトからは数値トークンを使う一括実行を基本とする。対話REPLの画面出力を機械用プロトコルとは扱わない。状態保存・復元や常駐プロトコルは今回追加しない。

### 4.15 CLIモジュール

CLI型・構文・シーケンスは第9章を正本とする。mainは接続だけ、argsは起動引数、parserは構文、repl/batchは逐次実行、outputは整形とI/O、errorはCLI失敗分類。計算式を独自に評価しない。

## 第5章：RPNスタック設計

### 5.1 固定4段の基本操作（SPEC-STACK）

以下でA=X、B=Y、C=Z、D=T、L=LAST X、r=結果。全段は常に値を持ち、空スタック・underflowは存在しない。Tの複製は意図したRPN動作。

| 操作 | 実行前 `[X,Y,Z,T]; LAST X` | 実行後 | Lift/入力への効果 |
|---|---|---|---|
| lift | `[A,B,C,D]; L` | `[A,A,B,C]; L` | 低水準関数では変更なし |
| drop_stack | 同上 | `[B,C,D,D]; L` | 同上 |
| swap_xy | 同上 | `[B,A,C,D]; L` | 同上 |
| roll_up | 同上 | `[D,A,B,C]; L` | 同上 |
| roll_down | 同上 | `[B,C,D,A]; L` | 同上 |
| set_x(V) | 同上 | `[V,B,C,D]; L` | 同上 |
| save_last_x | 同上 | `[A,B,C,D]; A` | 同上 |
| 二項演算 | 同上 | `[r,C,D,D]; A`、r=B op A | 成功時Idle/Enabled |
| 単項演算 | 同上 | `[r,B,C,D]; A`、r=f(A) | 成功時Idle/Enabled |
| ENTER | 同上 | `[A,A,B,C]; L` | 入力確定、Idle/Disabled |
| CLx | 同上 | `[0,B,C,D]; L` | 入力を破棄、Idle/Disabled |
| Recall V / Lift=Enabled | 同上 | `[V,A,B,C]; L` | Idle/Enabled、暫定 |
| Recall V / Lift=Disabled | 同上 | `[V,B,C,D]; L` | Idle/Enabled、暫定 |
| 全エラー | 同上 | 完全に同じ | phase/lift/buffer/flags/registers/configも同じ |

ENTER・CLxの通常動作はSRC-03を継承する。特殊なキー履歴による動作はTBD-001。IntegerのCHSは入力を確定して補数を生成しLAST Xを保存する。FloatのCHSは別分類（§5.2）。低水準のswap/rollとCommand::SwapXY等は区別し、後者は入力を確定してLiftをEnabledにする（SRC-08印刷p.99～100）。

### 5.2 LAST Xと命令効果

以下の入力終了列はHpInputSessionの責務、数値・LAST X・Lift効果はCoreの責務。phaseを伴う行はセッション合成動作を示す。

| 命令群 | 入力の扱い | LAST X | 実行後Lift | 仕様状態 |
|---|---|---|---|---|
| 四則/RMD/SQRT/論理/シフト/単項回転/CountBits | 確定 | 演算直前X | Enabled | LAST X分類確定SRC-08 p.100、数値細部TBD-007/008 |
| RotateN | 確定、abs(X)を回数として取得 | 元の回数X（絶対値化前、対象Yではない） | Enabled | 外部確認SRC-08 p.49/97/100 |
| CHS：Integer、任意phase | 編集中なら確定 | 演算前X | Enabled | 外部確認SRC-08 p.44/99/100。ゼロ時F5細部はTBD-007 |
| CHS：Float、Editing | 仮数/指数の符号編集を継続 | Keep | Keep | 外部確認SRC-08 p.99/100、編集細部TBD-017 |
| CHS：Float、Idle | 現在Xの符号を反転 | Keep | Enabled | 外部確認SRC-08 p.100の分類から導出 |
| ABS | 確定 | 演算前X | Enabled | HP標準機能、外部確認SRC-08 p.44/100 |
| MaskLeft/Right、EditBit | 確定 | 演算前X | Enabled | 独自引数API、暫定TBD-010 |
| ENTER | 確定 | Keep | Disabled | 継承TBD-001 |
| CLx | 確定せず破棄 | Keep | Disabled | 継承TBD-001 |
| SwapXY / RollUp / RollDown | 確定 | Keep | Enabled | 外部確認SRC-08 p.99/100 |
| SetRadix | 旧基数で確定 | Keep | Keep | SRC-03内の曖昧さを解消、互換性TBD-001 |
| SetWordSize | 旧設定で確定 | 必要時値変換、履歴保存なし | Keep | 独自直接API、領域変換はTBD-004 |
| SetSignedMode | 旧設定で確定 | Keep、raw保持で再解釈 | Keep | 外部確認SRC-08 p.35/98～100、Integer内の変更 |
| SetNumericMode | 旧モードで確定 | 数値変換（クリアしない） | Keep | 暫定TBD-006 |
| Store | 確定 | Keep | Enabled | 暫定TBD-011 |
| Recall / RecallLastX | 確定後、現在Liftで格納 | Keep | Enabled | 暫定TBD-011/003 |
| SetFlag | 確定 | Keep | Keep | 独自直接API、実機分類TBD-007 |
| ClearStack | 編集破棄 | 0 | Disabled | 独自拡張。4段とLAST Xを0、flagsは保持 |
| ClearRegisters | 編集も保持 | Keep | Keep | 独自拡張。記憶領域だけ0 |
| Push | Idleが前提 | Keep | Enabled | 独自拡張、確定した数値を1個入力 |

失敗時はこの表の効果を一切適用しない。RecallLastXはリフト前にLをコピーしてから格納する。Storeの対象値は入力確定後X。LAST Xへの保存は演算の成功確認後だけ。FloatのCHSはLAST Xを保存しない（SRC-08印刷p.100脚注）。

#### 5.2.1 LAST X分類の確定範囲

現在のCommand集合でHP対応命令に分類漏れはない。IntegerのAdd/Subtract/Multiply/Divide/Remainder/ChangeSign/Absolute/Sqrt、And/Or/Xor/Not、Shift全種、Rotate全種、CountBitsは実行前Xを保存する。SwapXY/RollUp/RollDown/Enter/ClearX/Store/Recall/RecallLastX/SetRadix/SetSignedMode/SetFlagは履歴保存をしない。FloatのChangeSignだけはSave対象外。Floatの四則/Absolute/SqrtはSave、Float Remainderは独自Save契約。BackspaceはKeep。独自PushはKeep、ClearStackはClear、ClearRegistersはKeep。独自SetWordSize/SetNumericModeは第4章の変換契約を使い、HPのWSIZE/FLOATと同一視しない。Mask/EditBitの引数付きAPIのSaveも独自契約。HP WSIZEは元X保存、HP FLOATへの移行はClearである（TBD-004/006でAPI対応を検討）。

### 5.3 演算シーケンス（SPEC-TXN）

入力終了が必要ならHpInputSessionが候補内でfinish→ReplaceX、続いてCore演算を実行する。成功時のみCoreと編集を一括置換。Core単体は入力確定処理なし。シーケンス図は[IA §5](rdp-calc-input-api-design.md)。

## 第6章：数値入力状態管理

### 6.1 状態と正本（SPEC-INPUT）

StackLiftはCore、EntryPhase/buffer/partはHpInputSession。文字列は編集の正本、Core Xは数値プレビュー。Core単独利用では完成値として扱う。所有・原子性は[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §3・4。

### 6.2 状態遷移図

[SI設計 §5](rdp-calc-stack-input-detailed-design.md)の直交2機械をセッション全体の状態として維持する。

### 6.3 状態遷移表

旧TR-01～21はSI-01～21に対応。正本は[SI §5・10](rdp-calc-stack-input-detailed-design.md)。成功時の値・Liftを維持し、編集ガードの所有元を移す。

### 6.4 PushとCLI一括入力

Core Pushは完成Numberを厳密検査し条件lift後Enabled。Editing拒否はHpInputSessionでのみ行う。CLIは正規化済みNumberを渡す。キー開始Start・継続ReplaceXはLift中立。ENTER後の完成値は上書き後Enabled、キー入力は上書き後Disabledを保持。

## 第7章：整数演算設計

### 7.1 値域と中間精度（SPEC-INT）

W=1..64、M=2^W−1、S=2^(W−1)、a=Yのraw bits、b=Xのraw bits。

| 形式 | 数学値の範囲 | ゼロ |
|---|---|---|
| Unsigned | 0～M | raw0のみ |
| OnesComplement | −(S−1)～S−1 | +0=0、−0=M |
| TwosComplement | −S～S−1 | raw0のみ |

W=1の1の補数には±0しか存在しない。ゼロ除算の検査はraw==0だけではなく両方のゼロを対象とする。ビット演算は両ゼロを異なるビット列として扱う。

Unsignedの乗算は最大(2^64−1)^2を扱うためu128を使う。これをi128に変換しない。符号付き数学値の加減乗除はi128で足りる。減算の符号なし中間差はi128に変換可能。Carryのraw加算・積はu128。幅64に対して`1u64 << 64`を行わない。数値解釈の符号拡張と、ワード幅変更を混同しない。SetWordSizeによる幅拡大はゼロ拡張である。W'=64でも64bitシフトを避ける。

### 7.2 演算アルゴリズムの規定

1. **ADD**：Unsigned/2の補数はraw和の低Wビット。Carryはraw和>M。1の補数はraw和の低Wビットに上位Carryを加え戻す。Carryは補正前のraw和で判定。Gは解釈済み数学値の和が形式の範囲外か。
2. **SUB**：Unsigned/2の補数は(a−b) mod 2^W。Carryはraw比較a<b。1の補数は借り発生時に低Wビット差からさらに1を減じ、Wビットへ収めるend-around borrowを暫定採用。数学差でGを判定。SUBをADD(Y, CHS(X))へ置換しない。ゼロ表現はTBD-008。
3. **MUL**：数学積を求めGを判定。Unsigned/2の補数は低Wビット。1の補数は絶対値の積をMで剰余化し、負ならそのWビット反転。ゼロ結果の符号はオペランドの符号bitの排他的論理和を暫定採用。W=1でもM=1で除算可能。CarryはKeep。
4. **DIV/RMD**：商は0方向への切捨て、余りr=y−q*x。Unsignedはu128、符号付きはi128。2の補数最小値/−1もi128で計算し、DIVは結果を幅内へ戻しG=1。負のゼロ除数もDivisionByZero。1の補数のゼロ商は符号bitのXOR、ゼロ余りはYの符号bitを暫定継承。
5. **符号反転/ABS**：数学値でGを判定。2の補数の最小値はrawを維持しG=1。UnsignedのCHSは0なら0、非0ならmod 2^Wで負数を表しG=1とする暫定演算（完成数値の負入力拒否とは異なる）。1の補数のCHSはraw反転、ABSは符号bitを見て負なら反転し−0を+0へ。
6. **SQRT**：非負整数の最大k（k²≤x）を整数演算で計算。Carryはk²≠x。負値はDomainError、1の補数−0は0として計算し暫定+0。

これらは暫定実装を可能にする具体式。特に1の補数のゼロ結果と範囲外ビット列はTBD-008、各命令のflagsはTBD-007。数学値だけに変換してrawを捨てる実装は禁止。

### 7.3 フラグ記法

`Cond(p)`はp=trueならSet、falseならClearであり、前の値に関係なく更新する。`Set`/`Clear`は無条件、`Keep`は旧値保持。更新対象外のF0～3は常にKeep。表のLX=Saveは演算直前X保存、EはLift Enabled。すべて成功時のみ反映。エラーは全flagsを保持。

### 7.4 算術・論理命令表

| 命令 | オペランド | 演算内容 | 結果 | Carry | Out-of-range | LAST X | Stack Lift | 仕様状態 |
|---|---|---|---|---|---|---|---|---|
| ADD | Y,X | y+x、1補数は循環Carry | X=r、Drop | Cond(raw和>M) | Cond(数学和範囲外) | Save | E | 継承、細部TBD-007/008/003 |
| SUB | Y,X | y−x、循環Borrow | X=r、Drop | Cond(a<b) | Cond(数学差範囲外) | Save | E | Borrow意味は外部確認SRC-07、残り暫定 |
| MUL | Y,X | y*x | X=r、Drop | Keep | Cond(数学積範囲外) | Save | E | SRC-03継承、TBD-007/008 |
| DIV | Y,X | trunc(y/x) | X=商、Drop | Cond(余り≠0) | Cond(商範囲外) | Save | E | Carry・2補数例外は外部確認SRC-07、ゼロ等暫定 |
| RMD | Y,X | y−trunc(y/x)*x | X=余り、Drop | Keep | Keep | Save | E | flagsはSRC-08 p.98、LAST Xはp.100。1補数ゼロ表現TBD-008 |
| SQRT | X | floor(sqrt(x)) | X=r、他段保持 | Cond(r²≠x) | Keep | Save | E | 継承＋暫定TBD-007/008 |
| CHS（Integer、全phase） | X | 現形式の補数を生成 | X=r、他段保持 | Keep | Cond(結果範囲外) | Save | E | 分類確定SRC-08 p.44/98/100、UnsignedゼロF5はTBD-007 |
| ABS | X | 絶対値 | X=r、他段保持 | Keep | Cond(結果範囲外) | Save | E | HP標準命令、SRC-08 p.44/98/100 |
| AND | Y,X | a & b | X=r、Drop | Keep | Keep | Save | E | 継承、Appendix照合TBD-007/003 |
| OR | Y,X | a OR b | X=r、Drop | Keep | Keep | Save | E | 同上 |
| XOR | Y,X | a XOR b | X=r、Drop | Keep | Keep | Save | E | 同上 |
| NOT | X | (~b)&M | X=r、他段保持 | Keep | Keep | Save | E | 同上 |

Floatの入力中CHSだけはTR-09を使い、Integerの入力中CHSは本表を使う。ゼロ除算でCarryやGを新たにSetしない。RMDの「Keep」はGを0に戻す意味ではない。MUL後のCarryも以前の値を保持する。

#### 7.4.1 Appendix A照合結果

SRC-08印刷p.98の一覧で、整数F4/F5更新の対象分類を確認した。RMD、AND/OR/XOR/NOT、MASKL/MASKR、SB/CB、B?、#B、LJはこの更新一覧に含まれず両flagsを保持する。ADD/SUB/DIVはC/Gを更新、MULはC保持/G更新、SQRTはC更新/G保持、ABS/CHSはC保持/G更新、シフト・回転はC更新/G保持。DBL×はC保持/Gクリア、DBL÷はC更新/Gクリア。TBD-007は1補数の境界値、UnsignedのCHS(0)等の条件評価とFloatのG方針だけを残す。対象分類が確定しても全数値境界を確認済みとはしない。

### 7.5 シフト・回転

SL: `(x<<1)&M`、SR: `x>>1`、ASR: SignedModeがUnsignedなら`x>>1`、それ以外は`(x>>1) | (元MSB*S)`。符号付き形式だけ最上位bitを複製する（SRC-08印刷p.47）。RLは押し出しMSBをLSBへ、RRはLSBをMSBへ。RLC/RRCは旧Carryを空いた端へ挿入し、押し出しbitを新Carryへ。u64の固定64bit回転をWbit回転の代わりに使わない。

| 命令 | オペランド | 演算内容 | 結果 | Carry | Out-of-range | LAST X | Stack Lift | 仕様状態 |
|---|---|---|---|---|---|---|---|---|
| SL | X | 左1bit、0挿入 | X=r | Cond(元MSB=1) | Keep | Save | E | 外部確認SRC-08 p.47～49/98/100 |
| SR | X | 論理右1bit | X=r | Cond(元LSB=1) | Keep | Save | E | 同上 |
| ASR | X | 符号付きはMSB複製、Unsignedは0挿入 | X=r | Cond(元LSB=1) | Keep | Save | E | 同上 |
| RL | X | Wbit左循環 | X=r | Cond(元MSB=1) | Keep | Save | E | 同上 |
| RR | X | Wbit右循環 | X=r | Cond(元LSB=1) | Keep | Save | E | 同上 |
| RLC | X,F4 | Carry経由左循環 | X=r | Cond(元MSB=1) | Keep | Save | E | 同上 |
| RRC | X,F4 | Carry経由右循環 | X=r | Cond(元LSB=1) | Keep | Save | E | 同上 |
| RLn | Y,X=n | 左回転abs(n)回 | X=r、Drop | n=0はKeep、他Cond(最後のbit=1) | Keep | 回数X | E | 外部確認＋0回は文書導出、TBD-009 Closed |
| RRn | Y,X=n | 右回転abs(n)回 | X=r、Drop | 同上 | Keep | 回数X | E | 同上 |
| RLCn | Y,X=n,F4 | Carry経由左abs(n)回 | X=r、Drop | 同上 | Keep | 回数X | E | 同上 |
| RRCn | Y,X=n,F4 | Carry経由右abs(n)回 | X=r、Drop | 同上 | Keep | 回数X | E | 同上 |

RotateNは元Xを現在の符号形式でi128へ解釈し、u128で絶対値を求める。負数も受理し、負のゼロは0と解釈する。通常回転はn=0..W、Carry経由はn=0..W+1だけ受理し、それを超えればInvalidCount（HP Error 2）。W=64のCarry経由は65回を受理する。n=0も二項命令としてDrop、元XをLAST X保存、Lift有効化しCarryを保持する。これはp.49の0回反復およびp.97/100の分類を組み合わせた導出で、実機観測ではない。n=Wで結果が元に戻ってもCarryをKeepにしてはならず、最後の1回の出力で更新する。効率化する場合も回数を単純にmod WへしてCarryを失わない。W=64のCarry付き輪は65bitなのでu128、または1回操作の反復を使う。

### 7.6 マスク・ビット操作・拡張命令

| 命令 | オペランド | 演算内容 | 結果 | Carry | Out-of-range | LAST X | Stack Lift | 仕様状態 |
|---|---|---|---|---|---|---|---|---|
| MaskLeft(n) | API引数0..W | 上位n個を1、他0 | X置換 | Keep | Keep | Save | E | 独自引数API、HP MASKLはTBD-010 |
| MaskRight(n) | API引数0..W | 下位n個を1、他0 | X置換 | Keep | Keep | Save | E | 同上 |
| EditBit(Set) | X,index | Xのbit indexを1 | X置換 | Keep | Keep | Save | E | 独自引数API、HP SBはTBD-010 |
| EditBit(Clear) | X,index | Xのbit indexを0 | X置換 | Keep | Keep | Save | E | 同上 |
| test_bit | X,index | bit index検査 | bool返却、X保持 | Keep | Keep | Keep | Keep | 独自照会API、実機B?の条件スキップは対象外 |
| CountBits | X | 1の個数 | X=個数 | Keep | Keep | Save | E | 独自API、互換対応TBD-010 |
| SetFlag(4,true) | API引数 | F4を1 | 値保持 | Set | Keep | Keep | Keep | 独自直接API |
| SetFlag(4,false) | API引数 | F4を0 | 値保持 | Clear | Keep | Keep | Keep | 同上 |
| SetFlag(5,true/false) | API引数 | F5変更 | 値保持 | Keep | Set/Clear | Keep | Keep | 同上 |
| LJ | 未確定 | 左寄せ | 未確定 | TBD | TBD | TBD | TBD | 未対応TBD-014 |
| DBL× | 未確定 | 倍長積 | 未確定 | TBD | TBD | TBD | TBD | 未対応TBD-014 |
| DBL÷ | 未確定 | 倍長除算 | 未確定 | TBD | TBD | TBD | TBD | 未対応TBD-014 |
| DBLR | 未確定 | 倍長剰余 | 未確定 | TBD | TBD | TBD | TBD | 未対応TBD-014 |

MASK n=0は0、n=WはM、それ以外だけシフト式を使う。indexは0がLSB。bit testのboolをXへ勝手に積まない。LJ/DBL系はSRC-03に存在するため将来項目として保持し、今回の演算APIで仮実装しない。

#### 7.6.1 実機の引数配置と現在APIとの差

SRC-08印刷p.50～51/97/100ではMASKL/MASKRはabs(X)を個数とする単項命令で、Xをマスクへ置換し元XをLAST Xへ保存する。SB/CBはYが対象、abs(X)がbit番号、Dropして結果をXへ置き元XをLAST Xへ保存する。現在の引数付きMaskLeft/Right、EditBitは独自APIであり、そのままHPキーへ割り当てない。HP形式の新Command追加はTBD-010。B?のプログラム条件スキップは対象外。#BはX内の1を数え、LAST Xを保存しXを個数に置換する。LJはXを左寄せし、無条件LiftでYへ置きXに移動数を返す（非0入力はp.47で確定）。LJ(0)とDBL系の全詳細はTBD-014に残す。

### 7.7 浮動小数点（SPEC-FLOAT）

f64同士のAdd/Subtract/Multiply/Divide/Remainder/ChangeSign/Absolute/Sqrtを扱う。Remainderは切捨て商によるRustの`%`相当。sqrtの負数はDomainError、±0除算はDivisionByZero、NaN/Inf入力または非有限結果はFloatingPointOutOfRange。有限のsubnormal/underflowで0になる結果は受理。演算はf64の精度で丸められ、十進丸めエミュレーションはしない。flagsはF4/F5ともKeepという独自暫定契約を維持する（TBD-016/007）。HPではFloat四則もF5を更新するため互換仕様ではない（SRC-08 p.98脚注）。スタック・Liftは整数の同分類と同じ。ただしFloatのChangeSignはLAST Xを保持し、編集中は編集を継続する。FloatのRemainderは独自拡張として演算前Xを保存する。

Floatモードでビット命令はNumericModeMismatch。整数/Float混在の自動昇格はしない。整数→float切替で情報欠落を拒否する厳密変換と、通常のf64計算時の丸めは別契約。変換チェックは2^64付近での`as u64`の飽和に依存せず、先に範囲を広い型で検証する。例えばUnsigned u64::MAXはf64で2^64へ丸められるため変換拒否する。

## 第8章：エラー処理設計

### 8.1 分類（SPEC-ERROR）

[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §4の全CalcError順位表と安全な検査依存関係を採用（CP-03 Resolved）。内部不整合最優先、計算不能・範囲・引数・mode順。入力文法はLiteralError/InputErrorへ分離。整数overflow成功＋flagsは維持。

### 8.2 原子性・復帰

executeはCore全状態、HpInputSession::applyはCore＋編集、CliSession::submitはさらにpendingを候補化し失敗で保持する。execute_allと通常CLIトークン列は先行成功保持。HP表示解除キーはTBD-012。Coreにエラー表示状態は追加しない。

## 第9章：CLI詳細設計

### 9.1 実行モードと型

- 引数に式なし：REPL。起動設定からCalculatorを1つ作り行間で保持する。
- 式あり：ワンショット。全体を1つの式文字列として解析し逐次実行、成功時Xだけstdoutへ1行出力。
- `--help`は終了0、`--word-size 0`等の不正オプションは終了2。
- 標準入出力はUTF-8を前提とする。数値とコマンドの受理文法はASCII。CLI内でstd::process::exitを乱用せずmainで終了コードを決める。

```rust
pub(crate) struct CliOptions {
    pub(crate) config: CalculatorConfig,
    pub(crate) expression: Option<String>,
}
pub(crate) enum ParsedAction {
    Command(Command), Literal(String), Keys(Vec<InputEvent>),
    TestBit(u8), ShowStack, ShowStatus, Help, Exit,
    Unsupported(String),
}
pub(crate) enum CliError {
    UnknownCommand(String), MissingArgument(String), InvalidArgument(String),
    LineTooLong,
}
pub(crate) enum CliRuntimeError {
    Syntax(CliError), Literal(LiteralError), Input(InputError), Calculation(CalcError), Io(std::io::Error),
}
pub(crate) struct Repl { session: CliSession }
```

全てCLI内のcrate型。CliOptions/ParsedActionにDefaultなし。ReplはCliSessionを所有し、I/Oはrunの引数として借用する。Literalは数値を確定せず文字列を保持し、その位置までのSetRadix等を実行した後にcli::parse_literalで変換する。`hex FF dec 10 +`を行頭の基数で一括変換しない。

### 9.2 関数仕様

```rust
// args.rs
pub(crate) fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliOptions, CliError>;
// parser.rs
pub(crate) fn parse_line(line: &str) -> Result<Vec<ParsedAction>, CliError>;
// repl.rs
impl Repl {
    pub(crate) fn new(calculator: Calculator) -> Self;
    pub(crate) fn run<R: std::io::BufRead, W: std::io::Write, E: std::io::Write>(
        &mut self, input: &mut R, output: &mut W, error: &mut E,
    ) -> Result<(), CliRuntimeError>;
}
// batch.rs（REPLも同じrun_actionsを使う）
pub(crate) fn run_batch<W: std::io::Write>(options: CliOptions, output: &mut W)
    -> Result<(), CliRuntimeError>;
// output.rs
pub(crate) fn render_x(snapshot: &InputSnapshot) -> Result<String, CalcError>;
pub(crate) fn render_stack(snapshot: &InputSnapshot) -> Result<String, CalcError>;
pub(crate) fn render_status(snapshot: &InputSnapshot) -> String;
// error.rs
pub(crate) fn exit_code(error: &CliRuntimeError) -> u8;
```

run_actionsはREPL表示と終了要求も処理するため、出力先と実行モードを受け取り、RunControlで制御結果を返す。

```rust
pub(crate) enum RunControl { Continue, Exit }
pub(crate) fn run_actions<W: std::io::Write>(session: &mut CliSession,
    actions: Vec<ParsedAction>, output: &mut W, interactive: bool)
    -> Result<RunControl, CliRuntimeError>;
```

| 関数 | 正常処理・前提 | 異常処理・状態変更 | 試験 |
|---|---|---|---|
| parse_args | base/width/signed/float/reg-count設定、式は最大1引数。helpはmainで別経路 | 引数欠落・重複矛盾・範囲外はSyntax、状態なし | TEST-053/054 |
| parse_line | 64KiB以内の行を全トークン化、命令引数構文検査。Literalは未評価 | 文法エラーでその行を全未実行、状態なし | TEST-053/060 |
| Repl::new | Calculator所有権を受領 | 計算状態を変更しない | TEST-054 |
| Repl::run | 行取得→parse→run_actions。空行は無処理。正常行でX表示 | Syntax/Calculationをstderrへ出して次行。Ioのみ終了Err | TEST-054/056 |
| run_actions | 順次Command実行、Literal→現在snapshot設定でparse→Push。Keysは順次HpInputSession::apply | 最初の失敗で停止し後続を未実行。先行成功保持 | TEST-048/053 |
| run_batch | 起動検証、計算、最終入力の完全性検査後にXをstdoutへ | 計算失敗は最終Xを出さずErr。呼出元がstderrに表示 | TEST-053/054 |
| render_x/stack | Snapshotからcore formatterで整形、stackはT/Z/Y/X順にラベル付き | 計算状態不変。入力中Xはbufferを併記 | TEST-043/056 |
| render_status | mode/radix/width/signed/F0～5/phase/lift/個数 | 不変 | TEST-055 |
| exit_code | Syntax/Literal→2、Input/Calculation→1、Io→3 | 不変 | TEST-054 |

run_actionsはinteractive=false時にShowStack/ShowStatus/Help/Exit/TestBitをInvalidArgumentとして実行前検査で拒否する。これによりワンショットstdoutを最終Xの1行に固定する。REPLでは照会の結果を表示し、Exitは以降を実行しない。Batchで最後にEditingが残った場合は、そのbufferをHP編集器のfinishで完全な数値として検査する。不完全なら終了1、成功ならその値をformat_numberで表示する（ENTERは挿入しない）。Batch出力には編集bufferを併記しない。line長は読み込み時に上限+1までに制限し、超過分を改行まで捨ててLineTooLongとして復帰する。

### 9.3 構文（SPEC-CLI）

起動オプションの引数は`--base {bin,oct,dec,hex}`、`--word-size N`、`--signed {unsigned,ones,twos}`、`--float`、`--register-count N`。数値オプションNは十進。`--float`と非DECの明示基数は構文エラーとして拒否する。

コマンド名は大文字小文字を区別しない。数値区切りはASCII空白。コメントはトークン境界の`#`以降、ただし引用符内を除く。任意のシェルコードを実行する機能は持たない。

| 入力 | 意味 |
|---|---|
| `rdp-calc` | REPL |
| `rdp-calc --base hex --word-size 8 --signed unsigned "FF 01 +"` | 8bit unsignedの一括入力、結果`0`、C=1/G=1 |
| `rdp-calc --base dec "5 ENTER 3 +"` | 明示ENTERを含む入力、X=8 |
| `rdp-calc --float "0.5 0.25 +"` | Float、結果0.75 |
| `hex / dec / oct / bin` | SetRadix |
| `wsize N` | SetWordSize、Nは十進1～64。HPのXを使うWSIZEではない |
| `signed unsigned / signed ones / signed twos` | SetSignedMode |
| `integer / float` | SetNumericMode |
| `+ * / %`、`add sub mul div rmd` | 算術 |
| `chs abs sqrt and or xor not` | 単項/論理 |
| `enter swap rollup rolldown lastx` | スタック |
| `sl sr asr rl rr rlc rrc` | 1回操作 |
| `rln rrn rlcn rrcn` | X=n、Y=対象値のRotateN |
| `maskl N / maskr N` | 引数方式、Nは十進0～64かつW以内 |
| `sb N / cb N / bit? N / popcount` | ビット操作、Nは十進index。bit?はREPLだけ |
| `sto N / rcl N` | 十進index、0始まり |
| `sf N / cf N` | SetFlag true/false、0～5 |
| `clx / clearstack / clearregs` | 各クリア |
| `keys "5 ENTER 3 +"` | キー入力。数値片をKey::Digit等へ展開し数値間に自動ENTERしない |
| `stack / status / help / exit` | REPL表示・終了 |

整数リテラルは`[+-]?[0-9A-Fa-f]+`、接頭辞`0b/0o/0d/0x`で基数上書き可（接頭辞は符号の後）。`0d`は独自十進接頭辞。接頭辞なしはその時点の基数。全基数でIA §2のビット列化とWordSize切り捨てを行う。Unsigned負数も受理しOnesの-0を維持する。

Floatは十進`[+-]?((digits(\.digits*)?)|(\.digits+))([eE][+-]?digits+)?`のみ。NaN/Inf、16進float、桁区切りunderscore、全角数字は受理しない。HEXの`E`は数字、FloatのEは指数。単独`-`はLiteralとして正0。減算はsubを使用する。予約コマンド名を優先し、そうでないトークンで全てHEX桁ならLiteral候補とする。不正数字はLiteralError::InvalidDigit、他の未知語はUnknownCommand。`G`など単独英字の数字誤記はUnknownCommandになるため、`0xG`のように数値接頭辞を付けたものは必ずLiteralとしてInvalidDigitへ送る。

keysは引用符内の命令名と数字片を展開し、`digit:X`、`.`、`eex`、`bs`、`chs`も受理する。キー数値片に符号/接頭辞は許可しない（符号はchs、基数は外側で設定）。`keys "1 2"`は12であり2個の数値ではない。keys内の`eex`だけがKey::Exponent、文字EはKey::Digit('E')。数値部の基数適否は実行時に検証する。keys内からkeysをネストさせない。

既知未対応語`lj, dblmul, dbldiv, dblr, wsize-x, run, label, goto, call, rtn`はParsedAction::Unsupportedにし、実行時UnsupportedOperation。未知語との区別をエラーメッセージで示す。全HPキー互換文法を完成したとは扱わない。

### 9.4 CLIシーケンスと実装窓口

[入力API設計 IA-1.0](rdp-calc-input-api-design.md) §3～5のCliSession::submitと図を正本とする。§9.1～9.3のI/O・起動引数・行構文は維持するがrun_actionsは`&mut CliSession`を受ける。Repl::newはCalculatorをセッション内へ渡す。Literalはparse_literal後Push、KeysはInputEventをapply。Batch末尾キー編集はHP finishで検査しCLI切り捨て規則を適用しない。表示はInputSnapshot.calculatorと編集欄を読む。終了コードはSyntax/Literal=2、Input/Calculation=1、Io=3へ更新する。

## 第10章：テスト設計

旧TEST-IDは維持する。下表のInputDigit等はKey::Digit等へ、編集を含む試験はHpInputSessionへ移設する。CLI parse試験はLiteralErrorへ分離し、期待値差分は[SI試験 §9](rdp-calc-stack-input-test-cases.md)が正本。

### 10.1 試験区分とオラクル

モジュール単体、コア結合、RPN状態遷移、整数境界値、flags、HP互換、CLIシナリオを分ける。次表の「暫定」は本書の暫定実装契約の試験であり、HP-16Cの未確定期待値を推測したものではない。HP互換試験では該当TBDが閉じるまで期待値を空欄/TBDにし、ignoredまたは別の未確定一覧として管理する。暫定試験の成功数をHP互換の成功数へ算入しない。

初期条件の省略時は新しい電卓、16bit/TwosComplement/HEX/全0/I_D。DECは明示的に設定する。`keys`はキー入力、Pushは完成数値、裸の数値はテスト設定値を意味する。結果表記は、rawと書いたものはビット列、それ以外は指定形式の数学値。A～Fを含む数は16進raw。表記のC/GはF4/F5。

| TEST-ID | 分類 | 対象機能 | 入力条件 | 期待結果 | 対応仕様 |
|---|---|---|---|---|---|
| TEST-001 | 単体 | WordSizeの値域 | 1..64、0、65 | 1..64は成功、0/65はInvalidWordSize | SPEC-DATA §4.7 |
| TEST-002 | 単体 | 幅マスク | W=1/8/16/32/64 | 1/FF/FFFF/FFFFFFFF/FFFFFFFFFFFFFFFF、panicなし | SPEC-INT §7.1 |
| TEST-003 | 単体 | 明示マスク | Word::from_bits(0x12A5,W8) | bits=A5 | SPEC-DATA §4.7 |
| TEST-004 | 単体 | 2補数解釈 | W8 raw80/FF/7F | 数学値−128/−1/127 | SPEC-INT §7.1 |
| TEST-005 | 単体 | Unsigned最大値 | W64 rawFFFFFFFFFFFFFFFF | to_signed=18446744073709551615 | SPEC-INT §7.1 |
| TEST-006 | 単体 | 1補数−0保持 | W8 rawFF、OnesComplement | rawFF、is_negative=true、is_negative_zero=true、数学値0 | SPEC-DATA §3.3 |
| TEST-007 | 境界値 | W1各形式 | raw0/1 | Unsigned=0/1、2補数=0/−1、1補数=+0/−0 | SPEC-INT §7.1 |
| TEST-008 | 単体 | 安全な生成 | from_integerの形式別最小/最大とその外側 | 境界内成功、外側IntegerOutOfRange | SPEC-DATA §4.7 |
| TEST-009 | 結合 | 他幅のWord取込み | W8へWord::from_bits(0x100,W16)をPush | IntegerOutOfRange、全状態保持 | SPEC-TXN §4.7 |
| TEST-010 | 単体 | Lift/Drop | [1,2,3,4]を独立に操作 | Lift=[1,1,2,3]、Drop=[2,3,4,4] | SPEC-STACK §5.1 |
| TEST-011 | 単体 | Swap/循環 | [1,2,3,4] | Swap=[2,1,3,4]、Up=[4,1,2,3]、Down=[2,3,4,1] | SPEC-STACK §5.1 |
| TEST-012 | 単体 | LAST X保存 | X=3,L=7、save_last_x | L=3、4段保持 | SPEC-STACK §5.1 |
| TEST-013 | 単体 | 固定段数 | 初期0でAdd | X=0、underflowなし | SPEC-STACK §5.1 |
| TEST-014 | 単体 | 回転逆操作 | 任意4段、roll_up→roll_down | 4段が元通り、L不変 | SPEC-STACK §5.1 |
| TEST-015 | 境界値/暫定 | Unsigned ADD | W8 Y=FF X=1 | X=0,C=1,G=1（暫定動作試験） | SPEC-INT §7.4; TBD-007 |
| TEST-016 | flags/暫定 | 符号付きoverflow | W8 2補数Y=7F X=1 | X=80,C=0,G=1（暫定） | SPEC-INT §7.4; TBD-007 |
| TEST-017 | flags/暫定 | 負数加算 | W8 2補数Y=FF X=FF | X=FE,C=1,G=0（暫定） | SPEC-INT §7.4; TBD-007 |
| TEST-018 | 境界値/暫定 | 1補数循環Carry | W4 ones Y=E X=3 | X=2,C=1,G=0（SRC-03例の暫定再現） | SPEC-INT §7.2; TBD-008/007 |
| TEST-019 | flags/暫定 | Borrow | W4 unsigned Y=0 X=1 | X=F,C=1,G=1（暫定） | SPEC-INT §7.4; TBD-007 |
| TEST-020 | 状態遷移/継承 | 基本加算 | DEC、keys 5 ENTER 3 + | [8,0,0,0],L=3,Idle/E | TR-01..08; TBD-001/003 |
| TEST-021 | 状態遷移/継承 | 連続ENTER | DEC、keys 5 ENTER ENTER 3 + | [8,5,0,0],L=3,Idle/E | TR-05/06; TBD-001 |
| TEST-022 | 状態遷移/継承 | CLx後上書き | DEC、keys 5 ENTER CLx 3 | [3,5,0,0],L=0,Editing/D | TR-07/02; TBD-001 |
| TEST-023 | 状態遷移/継承 | 演算後自動Lift | DEC、keys 5 ENTER 3 + 2 * | X=16、L=2、Idle/E | TR-01/08; TBD-001/003 |
| TEST-024 | 状態遷移/暫定 | LAST X呼出し | DEC、keys 5 ENTER 3 + LASTX | [3,8,0,0],L=3,Idle/E（暫定） | §5.2; TBD-003 |
| TEST-025 | 状態遷移/継承 | 4段循環 | DEC、keys 1 ENTER 2 ENTER 3 ENTER 4 ROLLDOWN | [3,2,1,4]、L=0（循環後Eは暫定） | §5.1; TBD-003 |
| TEST-026 | 状態遷移/暫定 | 基数中立 | DEC、keys 5 ENTER、hex、keys 3 + | X=8。基数変更前後Lift=D（暫定） | TR-13; TBD-001 |
| TEST-027 | 状態遷移/継承 | 演算後CLx | DEC、keys 5 ENTER 3 + CLx 7 | X=7、Y=0、L=3、Editing/D | TR-07; TBD-001 |
| TEST-028 | 結合 | Push連続 | DEC、Push(5),Push(3),Add | X=8、Idle/E。Pushごとに確定 | TR-17 |
| TEST-029 | 結合 | EditingとPushの混在 | InputDigit(5)後Push(3) | InputInProgress、buffer/stack/Lift保持 | TR-18 |
| TEST-030 | 結合/暫定 | STO/RCL | DEC、Push(5),Store(0),ClearX,Recall(0) | X=5、R0=5、L=0、Idle/E（暫定） | §5.2; TBD-011 |
| TEST-031 | 単体 | レジスタ境界 | 16個、get/set index15/16 | 15成功、16 InvalidRegister、状態保持 | §4.11 |
| TEST-032 | 単体 | 不正数字 | BINでInputDigit(2)、0xGをparse | InvalidDigit、不正開始でLiftしない | TR-21; §4.12 |
| TEST-033 | 状態遷移 | 入力確定失敗の原子性 | Floatでkeys 1 eex、Divide | IncompleteEntry、buffer=1eと全値保持 | SPEC-TXN; TR-21 |
| TEST-034 | 状態遷移/外部確認 | 整数編集中CHS | W8 twos DEC keys 5 CHS | X=-5、L=5、buffer=None、Idle/E。続く2はLiftして[X=2,Y=-5] | TR-10; SRC-08 p.44/99/100 |
| TEST-035 | 状態遷移/外部確認 | Idle CHS | Integer Push(5),ChangeSign / Float Push(5),ChangeSign、L=7 | Integer X=-5,L=5、Float X=-5,L=7、両方Idle/E | §5.2; SRC-08 p.100 |
| TEST-036 | 状態遷移/暫定 | Backspace | keys 1 2 BS BS | buffer=0、X=0、Editing、追加Liftなし（暫定） | TR-11; TBD-017 |
| TEST-037 | 単体 | buffer上限 | 受理可能な桁0を256個、さらに0 | 257文字目InputTooLong、256文字保持 | §6.4 |
| TEST-038 | 状態遷移/暫定 | 指数入力 | Float keys 1 . 5 eex CHS 2 ENTER | X=0.015、Y=0.015、Idle/D（暫定） | TR-04/05/09; TBD-017 |
| TEST-039 | 結合/外部部分確認 | 幅拡大 | W8 twos rawFFをスタックへ、SetWordSize(16) | 各スタックraw00FF。LAST X/記憶領域は独自暫定ゼロ拡張、互換試験対象外 | §4.4; SRC-08 p.32; TBD-004 |
| TEST-040 | 結合/外部部分確認 | 幅縮小 | W16 raw1234、SetWordSize(8) | スタックraw34。元幅へ戻しても1234を復元しない。記憶領域再配置は未確定 | §4.4; SRC-08 p.32/67; TBD-004 |
| TEST-041 | 結合/外部確認 | 補数変更 | W8 rawFFを3形式の全6遷移、C/G各0/1、Lift E/D | 全raw保持、解釈はUnsigned255/ones負0/twos負1、flags/Lift保持、LAST X履歴保存なし | SRC-08 p.35/98～100; TBD-005 Closed |
| TEST-042 | 結合/暫定 | 整数→Float情報欠落 | W64 unsignedでu64::MAXをXへ | SetNumericMode(Float)はIntegerOutOfRange、全状態保持（暫定） | SPEC-FLOAT; TBD-006 |
| TEST-043 | 単体/暫定 | 表示 | W8 rawFF、DEC twos/unsigned/ones、HEX F3=0/1でraw1 | DEC=-1/255/-0、HEX=1/01（暫定表示） | §4.12; TBD-018 |
| TEST-044 | flags/暫定 | MULのKeep | W8 unsigned Y=2 X=3、Cを0/1、G=1で各実行 | X=6、Cは元の値、G=0（暫定） | §7.4; TBD-007 |
| TEST-045 | 境界値/外部部分確認 | DIVとRMD | DEC Y=-7 X=3、twos、各独立 | DIV=-2,C=1。RMD=-1、符号はSRC-07。RMD flagsは暫定 | §7.2/7.4; TBD-007 |
| TEST-046 | 境界値/暫定 | 最小値/-1 | W8 twos Y=80 X=FF | DIV X=80,C=0,G=1、panicなし（暫定） | §7.4; TBD-007 |
| TEST-047 | 結合 | ゼロ除算の全状態保持 | 入力中0でDivide、既存L/flags/レジスタを非0 | DivisionByZero、全フィールド完全一致 | SPEC-TXN; §8.2 |
| TEST-048 | 結合 | 逐次バッチの失敗 | Push(5),Push(0),Divide,Push(9) | 5と0の成功は保持、Divide不変、9未実行 | §4.2/8.2 |
| TEST-049 | 単体/暫定 | 論理4種 | W8 Y=A5 X=0F、C/G各0/1 | AND=05、OR=AF、XOR=AA、NOT X=F0。C/G保持（暫定） | §7.4; TBD-007 |
| TEST-050 | 単体/外部確認 | 単項シフト回転 | W8 X=81,C=0、各独立、twos/Unsigned | SL02/SR40、ASR twos C0/Unsigned40、RL03/RRC40等。各C=1、G保持、L=81 | SRC-08 p.47/49/98/100 |
| TEST-051 | 境界値/文書導出 | 回転0/負数/上限 | W8 Y=81、X=0/-1/8/9、Carry経由9/10、W64 Carry経由65 | 0はDropし値/Carry保持、-1は1回と同じ。通常9はErr、Carry9成功/10 Err、65成功 | §7.5; SRC-08 p.49/97/100 |
| TEST-052 | 単体/暫定 | マスク/bit | W8 maskl3/maskr3、bit0 Set/Clear/test、count FF | E0/07、該当bitのみ変更、testは不変、count=8（暫定） | §7.6; TBD-010 |
| TEST-053 | CLI | ワンショットと途中基数 | --base hex、式 hex FF dec 10 + | stdout=265と改行、stderr空、終了0 | SPEC-CLI §9 |
| TEST-054 | CLI | 終了コード/復帰 | 正常/0除算/未知語/I/O失敗、REPLエラー後exit | 0/1/2/3。REPL計算エラー後の正常exitは0 | §8/9 |
| TEST-055 | flags | 番号と領域独立 | sf0..5、cf4、apply(C=Set,G=Keep)、index6 | F0..3不変、F4=1、F5保持、6はInvalidFlag | §4.10 |
| TEST-056 | CLI/暫定 | REPLキーと不完全入力 | float、keys 1 eex、stack、keys CLx、exit | 1eを編集中表示、CLxで復帰、終了0（表示細部暫定） | §6/9; TBD-017 |
| TEST-057 | 境界値 | u64最大乗算 | W64 unsigned (2^64−1)^2 | raw1、G=1、Carry保持（flagsは暫定）、中間overflowなし | §7.1/7.4; TBD-007 |
| TEST-058 | 境界値/暫定 | 1補数の負のゼロ | W8 ones、FF*01、00−01、除数FF | MUL FF、SUB FE（暫定）、除数FFはDivisionByZero | §7.2; TBD-008 |
| TEST-059 | 境界値/暫定 | SQRT | W64 unsigned最大値、16、負値−1 | 4294967295/C1、4/C0、DomainError（flags暫定） | §7.2/7.4; TBD-007 |
| TEST-060 | CLI | 構文と未対応 | 式 5 unknownword、式 5 dblmul | 未知語:行未実行/終了2。dblmul:5成功後未対応/終了1 | §9.3; TBD-014 |
| TEST-061 | 結合 | 設定・リセット | register_count0/1025、Float+HEX、生成後設定変更→reset | 不正生成を拒否。resetは生成時設定/全値0/F0..5=0/I_D | §3.2/4.2 |
| TEST-062 | 境界値 | f64エラーと許容値 | NaN/Inf Push、max*2、0.5+0.25、負sqrt | 非有限拒否、overflow拒否、0.75、DomainError | SPEC-FLOAT §7.7 |
| TEST-063 | 境界値/暫定 | W1とW64ビット操作 | W1 X=1をSL/RL、W64 mask64、index64 | SL0/C1、RL1/C1、mask=u64MAX、index64はInvalidBitIndex（回転は暫定） | §7.5/7.6; TBD-009 |
| TEST-064 | CLI/製品仕様 | 外部スクリプト自動化 | 外部からCLIを複数起動し式・設定を明示、正常/失敗を判定 | 同一プロセス内のみ状態継続、別起動は初期状態、stdout/stderrと終了コードを分離。内蔵プログラムAPIなし | SPEC-PROGRAM; §4.14; SRC-10 |
| TEST-065 | HP互換/保留 | 未検証命令と特殊状態 | TBD-001～015/017～020に対応する実機操作列 | 期待値はTBD、実機/Handbook照合後に登録。暫定値を転記しない | SPEC-STACK/INT/ADV; 第11章 |
| TEST-066 | HP互換/部分確認 | Handbook DIV/RMD | SRC-07、符号付きY=−7 X=3、Y=6 X=3 | DIV余り有C1/無C0、RMD非0符号はY。全スタック等の実機期待値はTBD | SRC-07; TBD-003/007 |
| TEST-067 | HP互換/保留 | 浮動小数点精度差 | 0.1+0.2、極大/極小、実機代表例 | f64自己試験は実施。実機比較の許容誤差はTBD-016、未設定で合格にしない | SPEC-FLOAT; TBD-016 |
| TEST-068 | flags/外部確認 | RMDフラグ保持 | W8 twos Y=7,X=4、C/Gを各0/1で独立実行 | X=3、C/Gとも実行前と同じ、L=4、Drop | SRC-08 p.98/100 |
| TEST-069 | 状態遷移/外部確認 | Idle BSP | X=5、L=7、Lift E/D、IdleでBackspace | X=0、L=7、Idle/D。他段/flags保持 | SRC-08 p.25/99 |
| TEST-070 | 状態遷移/外部確認 | Float編集CHS | Floatでkeys 5 CHS、L=7 | X=-5、編集中、L=7、Lift保持。続く2は-52 | SRC-08 p.99/100; 編集細部TBD-017 |
| TEST-071 | HP互換/将来受入 | 記憶領域再配置 | W16 R0=1234h,R1=5678h、WSIZE32→RCL0、WSIZE16→RCL0/1 | W32 R0=56781234h、元幅で1234h/5678h。現在のVec<Number>では満たさないため未実装 | SRC-08 p.67; TBD-004 |
| TEST-072 | HP互換/将来受入 | HP FLOAT変換 | Integer Y=25E47h,X=0でFLOAT | X=155207.0、Y/Z/T/L=0、W56、Lift E。独自SetNumericModeをこの期待値で試験しない | SRC-08 p.57; TBD-006 |
| TEST-073 | 単体/外部確認 | ABS | W8 twos X=FB / 80、Unsigned X=FB | twos FB→05、80→80/G1、Unsigned FB保持。C保持、L=実行前X | SRC-08 p.44/98/100 |

### 10.2 網羅条件と実行方法

- WordSizeは1～64の全幅を単体テスト。演算境界は1/2/4/8/16/32/63/64、各符号形式、最小/最大/0/±1/負のゼロ。
- 小幅1～4では全オペランド組合せを、実装と独立した数学式の参照モデルと比較する。参照モデルでもrawと負のゼロを保持する。HP未確認の式は暫定モデルと明記する。
- Keep命令は前flagsが0/1の両方、Cond命令は真偽両方。F0～3が変わらないことも比較する。
- 失敗試験はXだけでなく全Snapshotと内部EntryPartを比較。Floatは必要ならto_bitsで符号付きゼロを確認する。
- 公開APIの結合試験はcrates/rdp-core/tests、private関数の単体試験は各モジュール内。CLIはstdin/stdout/stderrと終了コードを検査する。
- `cargo test --workspace`で全体、`cargo test -p rdp-core`でコア、`cargo test -p rdp-cli`でCLI。必要な互換fixtureには資料版、印刷ページ、設定、操作列、期待状態、TBD解決日を持たせる。
- f64の演算結果は単純な正確値を除き、独立した参照値と絶対/相対許容誤差をケース別に持たせる。実機比較の許容値はTBD-016。NaNに対して通常のPartialEqを使わない。

### 10.3 先行テストとの対応

| 先行設計ID | 本書TEST-ID |
|---|---|
| SRC-02 WORD-01～04 | TEST-001～006 |
| SRC-02 ARITH-01～04 | TEST-015～019、047 |
| SRC-02 BIT-01/02 | TEST-049～052、063 |
| SRC-02 STACK-01～03 | TEST-010～014 |
| SRC-02 INPUT-01/02 | TEST-020～029 |
| SRC-02 CALC-01 | TEST-047/048 |
| SRC-02 CLI-01～03 | TEST-053/054/056 |
| SRC-03 ST-001～010 | TEST-020/021/022/023/024/025/026/027/047/015 |

## 第11章：未確定事項一覧

独立管理用の正本は[rdp-calc-tbd-list.md](rdp-calc-tbd-list.md)。以下は同版の同期コピー。着手前の判断とは「外部資料で確定する／暫定仕様を採用する／該当命令を後回しにする」の選択であり、全体の実装停止を意味しない。

| TBD-ID | 状態 | 内容 | 関連モジュール | 現在の暫定仕様 | 確認方法 | 確定条件 | 優先度 | 判断の期限・実装への影響 |
|---|---|---|---|---|---|---|---|---|
| TBD-001 | Partial | ENTER/CLx、初期Lift、基数変更の特殊履歴 | calculator,input | ENTER/CLxはD、基数変更は入力終了かつLift中立、BSP IdleはCLx相当を確定。起動Liftは独自Dを継続 | 原版p.25/99/100。通常キー遷移は確定、電源投入・reset後の実機Liftだけ未確認 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-002 | Resolved | CHSのEditing/Idle別効果とSRC-03内の矛盾 | calculator,input,arithmetic | Integerは入力終了・補数演算・SaveX・Lift E。Float Editingは符号編集/Keep、Idleは符号反転/Keep/E | 原版p.44/99/100。モード別分類を確定。UnsignedゼロのG条件はTBD-007、編集細部は017へ分離 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-003 | Resolved | LAST X対象命令の完全な分類 | calculator,command | §5.2.1で現Command全件をSave/Keep/Clearへ分類。Float CHSはKeep。HP FLOATはClear、WSIZEはSave | 原版p.100の一覧を全件照合。独自APIの契約は明示分離。TEST-034/035/070ほか | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-004 | Partial | ワード長変更の各領域の変換 | state,word,register | スタック縮小は低位保持、拡大はゼロ拡張。実機記憶領域はraw保存・境界再配置。現Vecは未対応 | 原版p.32/67。RegisterBank再設計、LAST X幅変換、HP WSIZE Commandの追加が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-005 | Resolved | 補数形式変更時の値とflags | state,word | Integerの全形式間でraw保持、数学値だけ再解釈。flags/Lift保持、LAST X履歴保存なし | 原版p.35/98～100、TEST-041。±0はビット列を保持。Float中の設定変更はTBD-006へ分離 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-006 | Partial | 整数/Float切替・LAST X・記憶領域 | state,number,calculator | HP FLOATはX=Y×2^X、Y/Z/T/L=0、W56、Lift E、記憶領域を数値変換しない。厳密全値変換とは異なる | 原版p.56/57。f64要件下の変換API分離・領域保存方式・逆変換の丸めを決定する | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-007 | Partial | 全命令のF4/F5更新規則 | flags,arithmetic,bitwise,calculator | Appendix A更新対象分類を確定。RMDはC/G Keep。Float四則は実機でG更新 | 原版p.44/98。1補数境界、Unsigned CHS(0)、独自f64とGの統合方針が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-008 | Open | 1補数の±0・範囲外結果・循環Borrow | word,arithmetic | raw保持、循環Carry/Borrow、MUL/DIVゼロ符号XOR、RMDゼロ符号Y | Section 4と小幅全組合せの実機/検証済み参照比較 | W=1、±0、範囲外を含むビット期待値確定 | High | 1補数互換実装前。Unsigned/2補数は先行可 |
| TBD-009 | Resolved | ASR、n回転、回数0/負/幅以上、Carry | bitwise,calculator | ASRはUnsignedでSR同等。n=abs(X)、通常0..W/Carry0..W+1、Y対象、Drop、元X保存、0回C保持 | 原版p.47/49/97/98/100。0回は0回反復と分類から導出、実機未観測。TEST-050/051/063 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | High | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-010 | Partial | MASKL/MASKR/SB/CB/B?のキー意味と配置 | bitwise,command | HP MASKはabs(X)の単項置換、SB/CBはY対象・abs(X)番号・Drop。B?の条件スキップは対象外 | 原版p.50/51/97/100。独自引数APIとHP Commandを分離する設計が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-011 | Partial | STO/RCLリフト、レジスタ数/番号 | register,calculator | STOは値保存でLift E、RCLは旧Liftに従い条件Lift後E、いずれもLAST X Keep | 原版p.25/99/100。固定16個・最大1024の独自規則と実機の番号/記憶領域境界との差が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-012 | Partial | HPエラー番号と解除キー動作 | error,calculator,CLI | HPは任意1キーでエラー表示を消して前表示へ戻す。コアは原子性を維持 | SRC-09印刷p.96。HP番号のUI写像、解除キーの消費/再実行は未確定。プログラムError4/5は対象外 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-013 | Partial | CLI超過入力とHPキー境界の分離 | cli,rdp-input | CLI部分Resolved（CP-01）。HPキーは厳密拒否を暫定維持、W≤3初回例外等は未採用 | IA §2、SI-H001/002/008、既存H32記録 | HP初回/継続桁・DEC符号bit・flags/Liftを確認 | High | HP互換キー入力ゲート。CLI正規化は実装可 |
| TBD-014 | Partial | LJ、DBL×/DBL÷/DBLR、実機WSIZE | arithmetic,calculator,command | HP WSIZE: abs(X)、0→64、Drop、元X保存。LJ非0: 左寄せ値をY、移動数をX、C/G Keep | 原版p.32/47/98/100。LJ(0)、DBL全結果配置/境界、HP WSIZEの実装APIを残す | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-015 | Resolved | プログラム記録・分岐・制限・実行API | CLI,docs | 対象外として確定。ProgramEngine等の候補型/APIを削除。自動化は外部スクリプト→CLI | ユーザー方針SRC-10。TEST-064を外部CLI自動化へ変更。HPプログラム互換試験は不要 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Low | 2026-10-25確認。実装時に文書期待値を検証する |
| TBD-016 | Open | f64と実機の精度・範囲・flags差の受容 | arithmetic,format | 有限f64、非有限拒否、subnormal受理、flags Keep | 実機代表例とf64の比較、用途別許容誤差レビュー | 対象例別の許容誤差と非互換一覧を承認 | Medium | f64基本演算は先行可。互換評価前必須 |
| TBD-017 | Partial | 浮動小数点編集・Backspace・指数符号 | input | Float CHSは編集継続・LAST X Keep、BSP IdleはCLx相当へ修正 | 原版p.25/99/100。仮数/指数の削除境界と入力中最終桁削除の全状態が残る | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-018 | Open | 表示桁・ゼロ埋め・実機表示窓 | format,CLI | F3で非DEC整数0詰め、Float最短往復表現、表示窓なし | 表示節、Wが桁数で割切れない場合の実機比較 | F3適用外を含む全基数の表示例確定 | Low | 基本CLIは先行可。表示互換判定前必須 |
| TBD-019 | Open | レジスタ演算、Iレジスタ、間接指定 | register,command | 未対応、直接STO/RCLのみ | Storage/Program節と対象機能レビュー | 命令集合・番号規則・flags・スタック影響確定 | Medium | 第2段階の追加機能実装前 |
| TBD-020 | Partial | HP-16C機能全体の到達範囲 | command,docs | 内蔵プログラミングを対象外、ABSをHP標準へ訂正。外部CLI自動化を正式用途とする | SRC-10/原版p.44。非プログラム命令の必須/将来/対象外の完全な棚卸しは継続 | Resolvedは仕様確定。Partialは右記の残件を継続。詳細は照合記録 | Medium | 2026-10-25確認。残件の判断前に完全互換と扱わない |
| TBD-021 | Open | Float BSで符号だけ残る境界 | rdp-input | -1からBSで符号だけ残る穴は未解決。CLI単独-→0（CP-02）とは別 | SI-H003、原版/実機の符号0・指数・点削除 | HP観測と製品のbuffer/X/phase/part/Lift期待値を確定 | High | 当該BS境界の実装前。一般編集は先行可 |
| TBD-022 | Resolved | エラー優先順位とvariant | core,rdp-input,cli | CP-03、IA §4の順位・依存ガード・型分離を採用 | IAとIA-C/IA-G試験 | 設計確定。実装時variant/payloadと全状態を検査 | Medium | 複合原因試験を必須化。実機番号は012に残る |

### 11.1 APIへの影響

TBD-004/006の記憶領域とモード切替はデータ構造の変更を要する可能性がある。その他の残件は原則として現在のCommand、純粋演算関数、メタデータの内部ルールを変更して対応する。例外としてTBD-010/014/019は実機特有のオペランド配置や間接指定を表す新Commandが必要になる。TBD-015は対象外としてClosed。TBD-020で追加命令を発見した場合は対象範囲とAPIを更新する。

## 第12章：実装順序

| 段階 | 対象モジュール | 実装内容 | 前提 | 完了条件 | 必要なテスト | TBDの影響 |
|---|---|---|---|---|---|---|
| 0 | docs、Workspace、lib.rs | 本書の暫定採用範囲を記録、2クレートと公開境界 | なし | 設計・対応/未対応一覧が一致 | 文書整合チェック | 001/002/013を基本入力の判断として記録 |
| 1 | config、error、word、number、flags | 安全な値型・幅・変換・更新指示 | 0 | 無効幅/型をResultで拒否、64bit境界安全 | 001～009、055 | 008のraw負ゼロ表現は保持 |
| 2 | stack、register、state | 固定段操作、初期状態、Snapshot | 1 | 4段/全領域の不変条件を保証 | 010～014、031、061 | 011の個数は暫定16で進められる |
| 3 | arithmetic | Unsigned/2補数四則、純粋評価、sqrt | 1 | 境界・ゼロ除算・通常overflowを区別 | 015～019、044～046、057、059 | 007のflagsは暫定表、008の1補数は別レビュー |
| 4 | format、rdp-input、CLI parser | リテラル・キー編集分離、phaseとCore Liftの協調 | 1 | 不正入力で既存buffer保持、Push契約明確 | 032～038、043 | 001/002/013/017/018。Float編集は後段でも可 |
| 5 | calculator | 原子的execute、execute_all、基本命令 | 2～4 | 全状態整合、CLIなしで利用可能 | 020～030、047/048 | 003のLAST Xは命令表へ隔離 |
| 6 | arithmetic、bitwise | 1補数、論理、回転、引数式bit/mask | 1/3/5 | 暫定仕様試験と未確定HP試験を分離 | 018/049～052/058/063 | 007/008/009/010を実装前レビュー |
| 7 | CLI一式 | 同一parserでREPL/Batch、I/O分離 | 5/6 | 出力・終了コード・途中基数変更を検証 | 053/054/056/060 | 独自CLI仕様は確定。実機キー対応は別 |
| 8 | state、arithmetic、format、input | Floatモード・厳密切替・幅/補数変更 | 2～7 | 全領域一括変換と失敗原子性 | 039～043、062、067 | 004/005/006/016/017 |
| 9 | compatibility fixtures、docs | Handbook/実機照合、TBD解決 | 5～8 | 実装対象命令の確認範囲・非互換点公開 | 065/066/067と閉じたTBDの追加例 | 不明な期待値は埋めない |
| 10 | register拡張、倍長/LJ | プログラミングを除く未対応命令の追加 | 9と該当詳細仕様 | 新命令ごとにAPI・状態・テスト確定 | 新規TEST-ID | 014/019/020。今回は実装対象外 |
| 11 | rdp-gui | Command入力とSnapshot表示 | 安定したコアAPI | CLIと同じ操作列で同じ状態 | コアシナリオ再利用 | UI固有エラー解除は012 |

ソースコードの実装はこの作業では行わない。各段階の「完了条件」は今後の実装時の受入条件であり、今回テスト実行済みという意味ではない。

### 12.1 設計整合性の確認記録

1. 型・フィールド表と概念クラス図の名称を照合。tupleの省略とvariantの図中表記は注記した。
2. Calculatorだけを外部の状態変更窓口とし、純粋評価と状態反映を分離した。
3. 公開APIに出現する型は本書内または標準ライブラリで定義。将来APIは明示的に未公開。
4. configを分離し、Radixを経由するerrorの循環を除去。programモジュールは対象外として削除した。
5. EntryPhase×StackLiftの4状態とTR表を対応させた。図で省略した自己遷移を本文で列挙した。
6. 命令群・境界・flags・状態・CLI試験にTEST-IDを付与し、先行試験との対応表を作成した。
7. 未検証のHP動作には暫定/TBDを付与し、HP互換試験の期待値を勝手に確定しなかった。
8. f64採用、非有限拒否、変換時の情報欠落検出を区別した。
9. GUIも公開Command/execute/snapshotを利用可能。CLI専用照会はコアに混入させない。
10. 全TBDに管理行と確認方法・確定条件を用意し、別ファイルと同期した。

文書内のMermaidは標準記法のみを用いる。確認の実施範囲と残る制約は設計判断記録の末尾に記す。

