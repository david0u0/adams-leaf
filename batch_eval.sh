cargo build --release --features batch-eval || { echo "編譯失敗" ; exit 1; }
# TODO: 執行 eval_scripts/ 中的所有腳本