# Instruction-level simulator for for CPUEX-Group2 computer

## コンパイラ係向け
実行環境は WSL2 を推奨します。  
まず、アセンブラが出力した `minrt.bin` をこの `README.md` と同じディレクトリに置きます。
また `make` および `cargo` がインストールされていることを確認してください。その状態で
```sh
$ make run
```
を実行すると、シミュレータが起動し、実行を開始します。`Makefile` を見ると何となくわかると思いますが、`make run` では `minrt.bin` に対して `./sld/contest.sld` を入力として与えて実行するようになっています。
シミュレータの実行中、 `minrt.ppm` (この `README.md` と同じディレクトリ)に画像が出力されていきます。またプログレスバーで実行の進捗が表示されます(現在までの出力の合計バイト数の、256x256 で最終的に出力されるべき合計バイト数に対する割合に基づいています)。

![progress bar](./screenshots/progress_bar.png)

面談ではシミュレータの実行終了後に `minrt.ppm` を見せればいいと思います。なお実行終了時にいくつかの統計情報のようなものが表示されますが、これは面談では関係ないと思います。
