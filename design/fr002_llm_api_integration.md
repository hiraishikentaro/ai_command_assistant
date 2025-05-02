# LLM API 連携機能（FR-002）詳細設計

## 1. 機能概要

LLM API 連携機能は、ユーザーから受け取った自然言語の指示を大規模言語モデル（LLM）の API に送信し、適切なコマンドを生成する機能です。この機能は外部の AI サービス（主に OpenAI API など）を利用して、自然言語からコマンドラインコマンドへの変換を行います。

## 2. 機能要件

### 2.1 基本要件

- 外部 LLM API（OpenAI API 等）への接続機能
- 効果的なプロンプト設計と管理
- API レスポンスの解析とコマンド候補の抽出
- 複数のコマンド候補の生成と管理
- エラーハンドリングとリトライ機能
- API 認証情報の安全な管理

### 2.2 コマンド生成のフロー

1. ユーザー入力を受け取り、LLM 用のプロンプトを構築
2. 適切なシステムプロンプトと共に API リクエストを構成
3. API にリクエストを送信し、レスポンスを待機
4. レスポンスを解析してコマンド候補を抽出
5. コマンド候補を整形して返却

## 3. 技術仕様

### 3.1 API リクエスト・レスポンスフロー

```
[入力プロセッサ] → [プロンプト構築] → [APIリクエスト] → [APIレスポンス] → [レスポンス解析] → [コマンド候補]
```

### 3.2 使用する API

- 主要 API: OpenAI API (GPT-4, GPT-3.5-turbo)
- バックアップオプション:
  - 将来的に Azure OpenAI Service
  - オープンソースモデル（ローカル実行や API）のサポート

### 3.3 プロンプト設計

#### 3.3.1 システムプロンプト

```
You are CommandGPT, a specialized AI assistant for generating command-line commands from natural language instructions.

Your task is to convert the user's natural language request into the most appropriate command-line command.

Follow these rules:
1. Generate shell commands for the user's operating system ({os_info}).
2. Generate a single command when possible, but use pipes or command chaining when necessary.
3. Provide brief explanations for complex commands.
4. Assess the safety level of each command (SAFE, CAUTION, DANGEROUS).
5. If a command is potentially destructive, suggest safer alternatives.
6. If the user's request is ambiguous, provide multiple command options if appropriate.
7. Format your response as valid JSON according to the specified schema.

Do NOT:
- Generate commands that could cause significant data loss without warnings
- Execute any commands yourself
- Include unnecessary verbosity in your explanations

Output JSON Schema:
{
  "commands": [
    {
      "command": "the shell command to execute",
      "description": "brief description of what the command does",
      "safety_level": "SAFE|CAUTION|DANGEROUS",
      "explanation": {
        "purpose": "main purpose of the command",
        "components": [
          {"part": "command part or flag", "explanation": "what this part does"}
        ]
      }
    }
  ],
  "is_ambiguous": boolean,
  "additional_questions": ["question to clarify ambiguity", ...]
}
```

#### 3.3.2 ユーザープロンプト構築

```
I need a {shell_type} command to {user_input}.
OS: {os_info}
```

### 3.4 データ構造

```rust
/// LLMのAPIリクエスト用設定
pub struct LlmRequestConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub timeout_seconds: u64,
}

/// APIリクエスト用のメッセージ
#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// APIリクエスト
#[derive(Serialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,
    pub response_format: ResponseFormat,
}

/// コマンド生成結果
pub struct CommandGeneration {
    pub commands: Vec<CommandCandidate>,
    pub is_ambiguous: bool,
    pub additional_questions: Vec<String>,
    pub raw_response: String,
}

/// コマンド候補
pub struct CommandCandidate {
    pub command: String,
    pub description: String,
    pub safety_level: SafetyLevel,
    pub explanation: CommandExplanation,
}

/// コマンドの説明
pub struct CommandExplanation {
    pub purpose: String,
    pub components: Vec<CommandComponent>,
}

/// コマンドの構成要素
pub struct CommandComponent {
    pub part: String,
    pub explanation: String,
}

/// 安全性レベル
pub enum SafetyLevel {
    Safe,
    Caution,
    Dangerous,
}
```

### 3.5 エラーハンドリング

- API 接続エラー：リトライ機能（バックオフ戦略）の実装
- タイムアウトエラー：ユーザーへのフィードバックと操作再試行の促し
- レスポンス解析エラー：エラーメッセージとフォールバック戦略
- API キー不正や認証エラー：適切なエラーメッセージと設定手順の表示

## 4. 実装方針

### 4.1 モジュール構成

```
src/
  ├── llm/
  │   ├── mod.rs           # モジュール定義
  │   ├── client.rs        # APIクライアント
  │   ├── prompt.rs        # プロンプト管理
  │   ├── request.rs       # リクエスト構築
  │   ├── response.rs      # レスポンス解析
  │   ├── config.rs        # 設定管理
  │   └── error.rs         # エラー定義
  └── command/
      ├── mod.rs           # モジュール定義
      ├── generator.rs     # コマンド生成メイン
      ├── candidate.rs     # コマンド候補モデル
      └── safety.rs        # 安全性評価
```

### 4.2 外部ライブラリ

- **reqwest**: HTTP 通信と API 接続
- **serde**: JSON シリアライズ/デシリアライズ
- **tokio**: 非同期処理
- **dotenv**: 環境変数管理（API キー等）
- **dirs**: ユーザーディレクトリの取得

### 4.3 環境変数管理

- API キーを環境変数または設定ファイルから取得
- 環境変数名: `ACIA_OPENAI_API_KEY`
- 設定ファイル: `~/.config/acia/config.toml`

### 4.4 テスト戦略

- 単体テスト: プロンプト構築、レスポンス解析機能
- 統合テスト: API リクエスト/レスポンスのモックを使用
- シナリオテスト: 典型的なユースケースでの end-to-end テスト

## 5. セキュリティ考慮事項

### 5.1 API キー管理

- API キーは環境変数または暗号化された設定ファイルに保存
- メモリ内での API キーの扱いに注意（ログ出力しない等）

### 5.2 データ送信

- ユーザー入力の送信前のサニタイズ
- 必要最低限の情報のみを API に送信
- 個人情報などセンシティブなデータの送信回避

### 5.3 危険なコマンドへの対応

- 潜在的に危険なコマンドへの適切な警告
- 危険度表示と代替コマンドの提案

## 6. 将来的な拡張性

### 6.1 複数の LLM プロバイダーサポート

- 異なる LLM プロバイダー（Anthropic, Gemini 等）に対応できる抽象化設計
- プロバイダー固有の設定やプロンプト最適化

### 6.2 ローカルモデルサポート

- オフライン環境でも動作するローカル LLM の統合
- ローカルモデルとクラウド API の切り替え機能

### 6.3 コマンドフィードバックループ

- 生成されたコマンドの実行結果を LLM に戻し、修正や改善を要求する機能
- ユーザーフィードバックによるプロンプト改善の仕組み
