# このリポジトリについて
[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)のCコンパイラをRustで実装しています。  

注意  
現段階ではC言語コンパイラと呼べるものにはなっていません。  
型はlong型しか使用できません。 
long型の四則演算, 比較演算, ビット演算, ポインタ演算, 制御構文(if-else, for, while, break, return), 関数, sizeofが可能です。  

使用できない構文
- long型以外の型
- グローバル変数
- シフト演算
- 関数ポインタ
- マクロ
- 7以上の引数  

その他もろもろ

#### 現在コンパイル可能なソースコード
```
long add(long a, long b) {
    return a + b;
}

long main(){
    long a;
    long b;
    long c;
    a = 10;
    b = 10;
    c = add(a, b)  - 10;
    return c;
}
```

# 目標
マクロ等を使用していないC言語プログラムのコンパイル(X64アセンブリの出力)を目標としています。  
コンパイラの書籍や文献を参考にしていないため、  
以下の点については考慮していません。    
- 最適化
- 最適なX86-64命令の使用
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
グローバル変数、ローカル変数の変数名とスタックフレームからのオフセット、  
変数の型を登録しておく必要があります。  

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
1. 生トークン作成
2. トークン作成
3. 抽象構造木化
4. アセンブリ出力

コンパイル中にエラーが発生した場合は、  
エラーが発生した箇所とその理由を表示して終了します。

### 1. 生トークン作成
ソースファイルのコメント以外を生トークンに分解します。  
生トークンはスペースまたは記号により分割されます。  

#### 生トークン情報
各生トークンは自身の文字列、タイプ、ソースファイル内の位置を情報として持ちます。

#### 生トークンのタイプ
生トークンは以下のタイプを持ちます。  
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

### 2. トークン作成
生トークンを抽象構造木を作成しやすい形(トークン)に変形します。  
現段階では記号タイプの生トークンに記号ラベルを、  
予約語の生トークンにはその予約語ラベルをつけます。

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
ローカル変数情報はキーをローカル変数名、  
値をその変数のスタックフレームオフセットと変数情報としたハッシュテーブルです。  
   
またローカル変数は各スコープで定義されている変数名のベクトルをベクトルとした、  
スコープ別ローカル変数ベクトルを持ち、  
スコープを抜ける際にこのベクトルを参照してローカル変数情報からローカル変数を削除します。

### 4. アセンブリ出力
作成した抽象構造木の内容をアセンブリにします。

# ファイル構造
各ファイルには以下の役割を持たせています。
|  ファイル名  |  役割  |  詳細  |  
| ---- | ---- | ---- |
|  main.rs  |  コンパイラバイナリ  | 引数として渡されたファイルをコンパイルします |
|  lib.rs  |  コンパイラテスト  | コンパイラの結合テスト用ファイルです |
|  token  |  トークン化  | ソースのトークン化を行います |
|  ast  |  抽象構造木作成  | トークンから抽象構造木を作成します |
|  output  |  アセンブリ作成  | 抽象構造木からX64アセンブリを出力します |
|  definition/  |  定義ファイル群  | コンパイルで使用する情報の定義ファイルがあります |
|  definition/functions.rs  |  関数定義  | 関数情報の定義を行います |
|  definition/numbers.rs  |  即値定義  | 即値情報の定義を行います |
|  definition/types.rs  |  型定義  | 型情報の定義を行います |
|  definition/variables.rs  |  変数定義  | 変数情報の定義を行います |
|  definition/symbols.rs  |  記号定義  | 記号情報の定義を行います |
|  definition/reservedwords.rs  |  予約語定義  | 予約語情報の定義を行います |


# テスト
テストは自動化しています。  
```
cargo test
```
で実行可能です。  

# デバッグ
ディレクトリにコンパイルしたいソースファイルを`test`というファイル名で保存し、  
VSCodeの`Debug executable`を実行してください。  
(デバッグにはVSCode拡張のrust-analyzerとCode-lldbが必要です。)

# 謝辞
[低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)を無償で公開してくださった作者に感謝申し上げます。  
コンパイラへの知識が深まる素晴らしいテキストでした。


# ライセンス
MITライセンスです。自由にコピー、改変して構いません。  
少しでもお役に立ったならばスターをしていただけると嬉しいです。  
アドバイス、コメントも大歓迎です。