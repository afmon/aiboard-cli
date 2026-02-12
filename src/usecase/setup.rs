use serde_json::json;

/// Generates the Claude Code hooks configuration JSON for aiboard integration.
/// Hooks into UserPromptSubmit, PostToolUse, and Stop events.
pub fn generate_hooks_json() -> serde_json::Value {
    json!({
        "hooks": {
            "UserPromptSubmit": [
                {
                    "matcher": ".*",
                    "hooks": [{
                        "type": "command",
                        "command": "aiboard hook ingest",
                        "async": true
                    }]
                }
            ],
            "PostToolUse": [
                {
                    "matcher": ".*",
                    "hooks": [{
                        "type": "command",
                        "command": "aiboard hook ingest",
                        "async": true
                    }]
                }
            ],
            "Stop": [
                {
                    "matcher": ".*",
                    "hooks": [{
                        "type": "command",
                        "command": "aiboard hook ingest",
                        "async": true
                    }]
                }
            ]
        }
    })
}

/// Returns the hooks configuration as a formatted JSON string.
pub fn generate_hooks_string() -> String {
    serde_json::to_string_pretty(&generate_hooks_json()).unwrap()
}

/// Generates the aiboard skill SKILL.md content for Claude Code integration.
pub fn generate_skill_content() -> String {
    r#"---
name: aiboard
description: エージェント間通信と会話ログの永続化を行う aiboard CLI
---

# aiboard スキル

aiboard は、AIエージェント間の情報共有と会話ログの永続化を行うローカルCLIツールです。
SQLite をバックエンドとし、スレッドベースでメッセージを管理します。

## いつ使うか

- **エージェント間通信**: 複数エージェントが共通のスレッドを介して情報を共有する場合
- **会話ログの永続化**: セッションを超えて会話履歴を保持・参照したい場合
- **セッション横断の情報共有**: 過去のセッションで得た知見や決定事項を後続セッションで参照する場合
- **外部会話の取り込み**: URL から会話内容を取得してローカルに保存する場合

## 基本フロー

```bash
# 1. スレッドを作成
aiboard thread create "設計相談"

# 2. メッセージを投稿
aiboard message post --thread <スレッドID> --content "認証方式はJWTで進めます"

# 3. メッセージを読み取り
aiboard message read --thread <スレッドID>

# 4. メッセージを検索
aiboard message search "JWT"
```

## hook 連携

`aiboard setup hooks --apply` を実行すると、Claude Code のフックに aiboard を登録できます。
登録後は以下のイベントが自動的にキャプチャされます:

- **UserPromptSubmit**: ユーザーの入力
- **PostToolUse**: ツール呼び出しの入力と応答
- **Stop**: セッション終了

## コマンド一覧

### メッセージ管理
- `aiboard message post --thread <id> --content <text>` - メッセージを投稿
- `aiboard message read --thread <id>` - スレッドのメッセージを読み取り
- `aiboard message search <query>` - メッセージを検索
- `aiboard message update <id> --content <text>` - メッセージを更新

### スレッド管理
- `aiboard thread create <title>` - 新規スレッドを作成
- `aiboard thread list` - スレッド一覧を表示
- `aiboard thread delete <id>` - スレッドを削除
- `aiboard thread fetch <url>` - URLから会話を取得して保存

### クリーンアップ
- `aiboard cleanup age <days>` - 指定日数より古いメッセージを削除
- `aiboard cleanup thread <id>` - スレッドとそのメッセージを削除
- `aiboard cleanup session <id>` - セッションの全メッセージを削除

## 注意事項

- ローカル専用ツールです。データはマシン上の SQLite ファイルに保存されます
- ネットワーク通信は `thread fetch` コマンドでの URL 取得時のみ発生します
- スレッドIDにはUUIDが使われます。短縮プレフィックスでの指定も可能です
- **クリーンアップ処理（cleanup）はユーザーの明示的な同意なしに実行してはいけません**。データの削除は不可逆な操作です
"#
    .to_string()
}
