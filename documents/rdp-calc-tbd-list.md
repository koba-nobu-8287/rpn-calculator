# rdp-calc 未確定事項一覧

文書版：v0.4 / 2026-10-19 / UTF-8・LF

[詳細設計書](rdp-calc-detailed-design.md) 第11章と同じ管理表。仕様の正本は詳細設計書の各章、TBDの管理表は本ファイルとする。変更時には第11章も同期する。現在は全項目Open。暫定仕様はHP-16C確認済み仕様ではない。

「着手前」は該当機能の実装前を意味し、プロジェクト全体を止めるものではない。未確定のまま進める場合は表の暫定案を実装対象として明示し、暫定試験だけを合格にする。完全互換の合格条件は未確定のまま保持する。

| TBD-ID | 内容 | 関連モジュール | 現在の暫定仕様 | 確認方法 | 確定条件 | 優先度 | 判断の期限・実装への影響 |
|---|---|---|---|---|---|---|---|
| TBD-001 | ENTER/CLx、初期Lift、基数変更の特殊履歴 | calculator,input | 初期Disabled、ENTERはLift→Disabled、CLxはX=0、基数変更は入力確定してLift保持 | Handbook Section 2/Appendix Bと操作列照合 | 連続ENTER・CLx・基数変更を含む全遷移の根拠/期待値記録 | High | 基本入力着手前に暫定採用を明示。互換確定は後続可 |
| TBD-002 | CHSのEditing/Idle別効果とSRC-03内の矛盾 | calculator,input,arithmetic | Editingは符号編集、Idleは単項反転・LAST X保存・Lift有効 | Appendix BおよびENTER/CLx後の実機操作 | 条件別X/LAST X/Liftと入力符号の表が確定 | High | CHS互換実装前。通常算術は先行可 |
| TBD-003 | LAST X対象命令の完全な分類 | calculator,command | 算術・論理・ビット演算で保存、交換/循環/Recallでは保持 | Appendix Bの全対象命令を照合 | Save/Keep/Clearと保存元を命令単位で確定 | High | メタデータは暫定実装可。互換合格前必須 |
| TBD-004 | ワード長変更の各領域の変換 | state,word,register | 全領域を縮小マスク/拡大符号拡張、F4/F5保持 | Handbook Word Size、実機幅変更 | スタック/LAST X/メモリ/flags/復元可否が確定 | High | 幅変更機能着手前に方式選択。固定幅演算は先行可 |
| TBD-005 | 補数形式変更時の値とflags | state,word | raw bits保持、数学値の再解釈、flags/Lift保持 | Complement Modesと実機照合 | 全形式間6遷移と±0の扱い確定 | Medium | 暫定実装後の互換検証まで保留可 |
| TBD-006 | 整数/Float切替・LAST X・記憶領域 | state,number,calculator | 全値の厳密変換、情報欠落拒否、LAST Xも変換、radix=DEC | Handbook Float変換とSRC-03のLAST Xクリア主張を照合 | 変換対象・丸め・破壊範囲・エラー条件確定 | High | 切替機能着手前。各モード単独生成は先行可 |
| TBD-007 | 全命令のF4/F5更新規則 | flags,arithmetic,bitwise,calculator | 第7章のSet/Clear/Keep/Cond表を仮採用 | Appendix A原表とSection 4、前flagsを0/1両方で試験 | 命令ごとに更新条件・例外・根拠を記録 | High | 仮実装可、互換リリース前必須 |
| TBD-008 | 1補数の±0・範囲外結果・循環Borrow | word,arithmetic | raw保持、循環Carry/Borrow、MUL/DIVゼロ符号XOR、RMDゼロ符号Y | Section 4と小幅全組合せの実機/検証済み参照比較 | W=1、±0、範囲外を含むビット期待値確定 | High | 1補数互換実装前。Unsigned/2補数は先行可 |
| TBD-009 | ASR、n回転、回数0/負/幅以上、Carry | bitwise,calculator | ASRはraw符号拡張、n=0..64、回数X対象Y、0でもDrop、最後のCarry | Section 4の各回転とAppendix A/B | 全幅と回数境界、オペランド配置、LAST X確定 | High | n回転実装前。1回操作は暫定可能 |
| TBD-010 | MASKL/MASKR/SB/CB/B?のキー意味と配置 | bitwise,command,program | 引数APIのみ具体化。test_bitは不変bool、実機条件スキップ未対応 | Handbook各命令とプログラム条件命令照合 | X/Y消費・結果配置・スキップ・flags確定 | Medium | 独自APIは先行、HPキー対応前必須 |
| TBD-011 | STO/RCLリフト、レジスタ数/番号 | register,calculator | 16個・0始まり、STO後E、RCLは条件Lift後E、個数1..1024 | Storage RegistersとAppendix B | 番号対応と両Lift状態での格納/呼出し結果確定 | Medium | 独自直接APIは暫定実装可 |
| TBD-012 | HPエラー番号と解除キー動作 | error,calculator,CLI | コア全状態保持、エラー表示はUI、CLxで入力取消 | Appendix Aと実機エラー復帰 | エラー番号・表示解除・キー消費の対応表確定 | Medium | コア実装を妨げない。互換UI前必須 |
| TBD-013 | 入力過大値とPush/キー入力の互換境界 | input,format,calculator | 入力範囲外は拒否、PushはIdleのみ・完了後E、演算overflowは成功 | 数字入力章と境界桁入力。拡張Pushは製品仕様レビュー | キー入力の桁超過規則確定、独自APIとの差を文書化 | High | CLI/キー入力結合前。暫定契約で先行可 |
| TBD-014 | LJ、DBL×/DBL÷/DBLR、実機WSIZE | arithmetic,calculator,command | 未対応。型の構造だけで結果を仮定しない | Handbook各命令のスタック図と例 | オペランド・結果・全flags・エラーを命令別に確定 | Medium | 該当拡張段階前。基本機能は先行可 |
| TBD-015 | プログラム記録・分岐・制限・実行API | program | 外部所有、1命令step候補、今回未公開 | プログラム章と製品スコープ判断 | 記録粒度、PC、ラベル、呼出制限、エラー型確定 | Low | 将来実装着手前必須。ProgramEngine API変更可 |
| TBD-016 | f64と実機の精度・範囲・flags差の受容 | arithmetic,format | 有限f64、非有限拒否、subnormal受理、flags Keep | 実機代表例とf64の比較、用途別許容誤差レビュー | 対象例別の許容誤差と非互換一覧を承認 | Medium | f64基本演算は先行可。互換評価前必須 |
| TBD-017 | 浮動小数点編集・Backspace・指数符号 | input | EntryPart分離、未完成指数のプレビューは仮数、確定時拒否 | Handbook入力節と操作列 | 編集中表示・削除・CHSの状態別期待値確定 | Medium | キー編集機能前、暫定契約を採用可 |
| TBD-018 | 表示桁・ゼロ埋め・実機表示窓 | format,CLI | F3で非DEC整数0詰め、Float最短往復表現、表示窓なし | 表示節、Wが桁数で割切れない場合の実機比較 | F3適用外を含む全基数の表示例確定 | Low | 基本CLIは先行可。表示互換判定前必須 |
| TBD-019 | レジスタ演算、Iレジスタ、間接指定 | register,command,program | 未対応、直接STO/RCLのみ | Storage/Program節と対象機能レビュー | 命令集合・番号規則・flags・スタック影響確定 | Medium | 第2段階の追加機能実装前 |
| TBD-020 | HP-16C機能全体の到達範囲 | command,program,docs | 主要整数機能優先。全命令互換は宣言しない | Appendix命令索引を機能一覧へ照合 | 必須/将来/対象外を全命令で明示し合意 | Medium | 完全互換表記をする前。現在の実装骨格は先行可 |

## 実装開始時に先に選択する項目

基本入力ではTBD-001/002/013、1の補数ではTBD-008、幅変更ではTBD-004、モード切替ではTBD-006、n回転ではTBD-009について「資料で確定する／本書の暫定動作で進める／当該命令を未対応にする」を実装記録へ残す。TBDだからといってWordSize、固定スタック、原子性、Unsigned演算、CLI構文の作業を止めない。

TBD-003/007は命令メタデータ・純粋演算関数内で差し替えられる。TBD-010/014/019は新しいCommandが必要になる可能性があり、現APIで実機命令を代用してはいけない。TBD-015だけは将来ProgramEngineのAPI自体が未確定。

## クローズ手順

1. 版と印刷ページを明記した一次資料または実機の設定・キー列・結果を保存する。
2. 期待X/Y/Z/T、LAST X、F4/F5、EntryPhase、StackLiftを必要な範囲で記録する。
3. 暫定仕様との差分、影響する関数とテストを列挙する。
4. 詳細設計書の該当節・命令表・状態表・図・試験表を一緒に更新する。
5. 外部確認済みと独自仕様を区別したままClosedとする。エミュレーターだけで確認した場合はその根拠と限界を残す。

TBDを閉じる際の追記欄：状態、確認日、資料版/ページ、観測手順、決定内容、関連TEST-ID、変更した設計判断ID。
