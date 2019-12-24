cargo build --release --features batch-eval || { echo "編譯失敗" ; exit 1; }
# TODO: 執行 eval_scripts/ 中的所有腳本
for file in $(ls eval_scripts | grep .sh)
do
    echo --- 開始執行 ${file} $1 ---
    sh eval_scripts/${file} $1
done