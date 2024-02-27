# Voicevox Engine（Rustのすがた）

axumとvoicevox_core-rsで実装。

releaseビルドは
- `target/release`にcdして
- `vvms`フォルダ作ってvvmを入れて
- `LD_LIBRARY_PATH="." ./rust-engine`
みたいにしないと動かないので注意。
