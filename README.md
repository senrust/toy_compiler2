# このリポジトリについて
[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)のCコンパイラをRustで実装しています。  

注意  
現段階ではコンパイラと呼べるものにはなっていません。  
非常に簡単な加減算の抽象構造木までを作成できる程度です。  

# 目標
マクロ等を使用していないC言語プログラムのコンパイル(X64アセンブリ出力)を目標としています。  
以下の点は一切考慮していないのでご注意ください。  
- 最適化
- 最適なX64命令の使用
- 全C言語機能の実装
- 規格を遵守した実装

# 背景
2921年3月頃からコンパイラの仕組みを理解しようと、  
[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)(以下Cコンパイラ作成入門)を読み進めながら、  
[toy_compiler](https://github.com/senrust/toy_compiler)(第一世代Cコンパイラ)を実装しました。
    
Cコンパイラ作成入門は大変分かりやすい内容ですが、   
20章以降から内容の飛躍があり、  
Cコンパイラ作成入門以降の内容の実装を自力で行おうとすると、   
コンパイラが以下の点を考慮した設計になっている必要があります。

1. 型情報  
型を考慮する必要があります。  
変数がどの型であるのか、演算がどの型を返すのか把握する必要があります。  

2. 変数管理  
グローバル変数、ローカル変数の変数名とその型を登録しておく必要があります。  

3. 関数宣言  
関数の関数名 引数、戻り値を把握できるようにする必要があります。

4. スタックフレームのサイズ  
スタックフレームのサイズを把握している必要があります。


Cコンパイラ作成入門に従って実装した第一世代Cコンパイラは、  
上記の点を考慮した設計ではないため、  
Cコンパイラ作成入門以上の内容を実装するには、設計の見直しが必要でした。  

また、当時はRustへの理解が浅く、非常に未熟なコードとなっていました。  
以上の点を踏まえた設計とRustらしい実装を行った本リポジトリを、  
第二世代コンパイラとしています。
  
# コンパイル処理手順
コンパイラは以下の順序で処理を行います。
1. トークン化
2. トークン解釈
3. 抽象構造木化
4. アセンブリ出力

コンパイル中にエラーが発生した場合は、  
エラーが発生した箇所とその理由を表示して終了します。

### 1. トークン化
ソースファイルのコメント以外をトークンに分解します。  
トークンはスペースまたは記号により分割されます。  

#### トークン情報
各トークンは自身の文字列、タイプ、ソースファイル内の位置を情報として持ちます。

#### トークンのタイプ
トークンは以下のタイプを持ちます。  
1. 識別子  
アルファベット、または_で始まる文字列

2. 数字  
0~9で始まりる文字列。数字トークンのみコンマでトークン確定を行いません。    
トークン化時点では有効な数字表現であるかのチェックは行いません。  
12A、0x91X、1.243.4なども有効なトークンとして扱われます。 
 

3. 記号  
C言語で有効な記号をトークンとします。  
複数記号のトークンも有効です。(++, &&, <<=など)  

4. 文字列  
'A'や"Hello world"などの引用符で囲われた領域です。

### 2. トークン解釈
トークンを抽象構造木を作成しやすい形(Node)に変形します。  
現段階では記号タイプのトークンにその役割ラベルを付けているだけです。

### 3. 抽象構造木化  
抽象構造木用情報(型、関数、変数情報)を作成してから、  
各Nodeを順番に処理することで、抽象構造木を作成します。    
途中関数の宣言や変数定義があれば抽象構造木用情報を更新していきます。

#### 抽象構造木
抽象構造木の節点にはそのタイプ(変数、演算、関数呼び出し等)の情報と、  
その節点の型情報とその節点の子として必要な抽象構造木を持っています。  
ある節点が加算の場合、  
左項と右項の抽象構造木を子として持ち、  
加算による出力の型がその節点の型情報となります。

#### 型情報
初期状態では型情報にはプリミティブ型のみです。    
構造体の宣言やtypedefで型情報を追加できます。  
型情報は型名、サイズ等(構造体の場合はメンバー)の情報を持っています。  
型情報テーブルは型名をキー、値を型情報としたハッシュテーブルです。  
型情報は構造体のメンバー、関数定義、変数定義、抽象構造木の各節点等で、  
必要となることから型情報の実体をRcで共有しています。  

#### 関数情報
宣言された関数の関数名と引数の数とその型及び返り値の型を情報として持ちます。  
関数情報テーブルは関数名をキー、値を関数情報としたハッシュテーブルです。  

#### 変数情報
宣言済みの変数の変数名とその型を変数情報テーブルとして持ちます。  

変数情報テーブルはグローバル変数情報とローカル変数情報の2種類を持っています。    
グローバル変数情報はキーをグローバル変数名、値を変数情報としたハッシュテーブルです。  
ローカル変数情報はキーをローカル変数名、値をその変数のスタックフレームオフセットと変数情報としたハッシュテーブルです。   
またローカル変数は各ブロックで定義されている変数名のベクトルをベクトルとしたブロック別ローカル変数ベクトルを持ち、  
ブロックを抜ける際にこのベクトルを参照してローカル変数情報からローカル変数を削除します。

### 4. アセンブリ出力
作成した抽象構造木の内容をアセンブリにします。

# ファイル構造
各ファイルには以下の役割を持たせています。
|  ファイル名  |  役割  |  詳細  |  
| ---- | ---- | ---- |
|  main.rs  |  コンパイラバイナリ  | 引数として渡されたファイルをコンパイルします |
|  lib.rs  |  コンパイラテスト  | コンパイラの結合テスト用ファイルです |
|  error.rs  |  エラー出力  | コンパイルできないソースに対するエラーを出力します |
|  tokenizer.rs  |  トークン化  | ソースのトークン化を行います |
|  token_interpreter.rs  |  トークン解釈  | トークンの解釈を行います |
|  ast_maker.rs  |  抽象構造木作成  | 抽象構造木作成 |
|  definition/  |  定義ファイル群  | コンパイルで使用する情報の定義ファイルがあります |
|  definition/functions.rs  |  関数定義  | 関数情報の定義を行います |
|  definition/numbers.rs  |  即値定義  | 即値情報の定義を行います |
|  definition/types.rs  |  型定義  | 型情報の定義を行います |
|  definition/variables.rs  |  変数定義  | 変数情報の定義を行います |



# 謝辞
[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)を無償で公開してくださった作者に感謝申し上げます。  
コンパイラへの知識が非常に深まる素晴らしいテキストでした。


# ライセンス
MITライセンスです。自由にコピー、改変して頂いて構いません。  
少しでもお役に立ったならばスターをしていただけると嬉しいです。  
ソースファイルへのアドバイス、コメントも大歓迎です。