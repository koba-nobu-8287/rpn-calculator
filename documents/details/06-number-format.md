# 06 数値表示

版：D-1.0 / 2026-11-14（日本時間）。実装用仕様、表示互換の暫定項目を含む。

## 1. 範囲・根拠・依存

Core formatは数値1個の文字列表現を返す。[全体設計](../rdp-calc-detailed-design.md)§4.12・7.7、01/02に従う。CLIのラベル・行構成・改行・出力先は11の責務。文字列からのparse、HP表示窓、端末I/Oは本単位に含めない。

```mermaid
flowchart LR
    SNAP[Snapshotの値と設定] --> F[Core format_number]
    F --> S[数値文字列]
    S --> O[11 CLIのラベル・改行・I/O]
```

## 2. APIと事前条件

`format_number(value: Number, radix: Radix, mode: NumericMode, config: &WordConfig, flags: &Flags) -> Result<String, CalcError>`を公開する。引数の借用形は全体設計が型を略記している部分の実装用具体化。値・mode・幅・有限性を検査し、無効値を表示のマスクで隠さない。Floatで非DECはNumericModeMismatch。複数原因は01の順位に従う。

関数は無副作用で、入力・計算状態を変更しない。失敗でも表示の代用値を返さずErrとする。

## 3. 整数

BIN/OCT/HEXは符号付き数学値でなくrawを非負として表示。DECはWordConfigの符号形式で解釈した数学値を表示し、Ones負0を`-0`とする。非DECで接頭辞・幅接尾辞を付けない。HEXの英字は大文字へ統一する（キー編集のASCII正規化と同じ表記）。

F3=Clearは先頭ゼロを省くが0は`0`。F3=Setでは非DECのみ幅から求めた桁数まで0詰め。BINはW、OCTはceil(W/3)、HEXはceil(W/4)。桁数が割り切れない場合も上位の有効bitを越える値を生成しない。DECではF3を無視する。

| W8、raw | 形式・基数 | F3=0 | F3=1 |
|---|---|---|---|
| 0x01 | HEX | 1 | 01 |
| 0x01 | OCT | 1 | 001 |
| 0x01 | BIN | 1 | 00000001 |
| 0xFF | Twos DEC | -1 | -1 |
| 0xFF | Unsigned DEC | 255 | 255 |
| 0xFF | Ones DEC | -0 | -0 |

## 4. Float

有限f64の最短往復可能表現を用いる。再解析で同じ有限値を得る表現とし、-0.0は`-0`、+0.0は`0`。F3、WordSizeによるゼロ埋めは適用しない。指数表記の採用閾値・末尾`.0`等、既存資料が完全一致を要求していない文字列へ新たな互換合否条件を設けない。実装の標準formatterを選び、代表出力を実装記録へ固定する。

HPの十進桁数・表示窓・丸めは再現しない。型一致・有限性を満たすsubnormalも表示対象とする。

## 5. 受入・残件

TEST-043、IA-L006とFloatのbit比較を参照。W1/3/4/8/63/64、全基数、F3の両値、0/最大raw/Ones負0を検査する。Floatは正負0・通常値・極小値の往復一致を確認。表示前後でSnapshotが同じであることを確認する。

TBD-018は製品仕様として確定。表示窓を設けず、W13・raw 1・F3=1はBIN `0000000000001`、OCT `00001`、HEX `0001`。W13の最大rawはHEX `1FFF`。Floatは同じf64へ再解析できることを符号付きゼロも含めて確認する。
