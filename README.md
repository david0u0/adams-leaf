_這一張文憑，仿佛有亞當、夏娃下身那片樹葉的功用，可以遮羞包醜。_
_<p align="right"> 錢鍾書《圍城》 </p>_

## 軟體需求 ##
主程式以 rust 語言編寫，推薦使用`rustc 1.38.0-nightly`，可以用`rustc --version`指令查看版本。

如果不確定是否符合需求，可以在專案根目錄用`cargo test`運行單元測試，通過的話應該就沒什麼問題。

## 專案架構 ##

### 文件 ###
所有文件（除了你正在看的 `README.md`） 都在 `doc/` 資料夾中。
建議閱讀順序：__README -> ACO演算法架構 -> 常數及自定義名詞 -> 實驗重現 -> 實作細節 -> 手動測試技巧__。

### 測試相關檔案 ###
#### test_case_generator.js ####
用 javascript 寫的小程式，用來動態生成測資。執行方式為：
```sh
node test_case_generator.js # 測資將直接從標準輸出流噴出來
```
實驗中所有測資皆以上述方法生成，以下稱靜態測資。
#### 靜態測資 ####
所有靜態測資都~~不負責任地~~直接散落在根目錄，即所有帶 `exp_` 或 `test_` 前綴的 json 檔案。

### evaluate.sh ###
把指令集成為一個 bash 檔，方便操作，其實是可有可無的東西。

### 實驗結果 ###
存放於 `exp_result/`，各項實驗之意義與設置詳見 `doc/實驗重現.md`。

### 程式碼 ###
所有程式碼皆使用 `rust` 語言編寫，主要為三個 TSN/AVB 路由演算法之比較，分別為：

- SPF: Shortest path first, 粗暴的最短路徑
- RO: Routing Optimization
    * 使用 `GRASP` 算法，是爬山法的一種變體，並非為動態演排程設計。
    * Laursen, Sune Mølgaard, Paul Pop, and Wilfried Steiner. "Routing optimization of AVB streams in TSN networks." ACM Sigbed Review 13.4 (2016): 43-48.
- ACO: Ant Coloy Optimization，即本專案開發的算法，詳見 `doc` 資料夾。    

## 單位 ##
- `時間單位` - 微秒 = 10^-6 秒
- `資料大小單位` - 位元組 byte
- `頻寬` - 位元組 / 微秒
