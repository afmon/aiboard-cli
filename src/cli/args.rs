use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aiboard", about = "エージェント間通信と会話ログの永続化")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// message の管理（投稿・読み取り・検索・更新）
    Message {
        #[command(subcommand)]
        action: MessageAction,
    },
    /// thread の管理（作成・一覧・削除・取得）
    Thread {
        #[command(subcommand)]
        action: ThreadAction,
    },
    /// hook イベントから会話ログを取り込む
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
    /// 古いデータのクリーンアップ
    Cleanup {
        #[command(subcommand)]
        action: CleanupAction,
    },
    /// hook と skill の設定
    Setup {
        #[command(subcommand)]
        action: SetupAction,
    },
}

#[derive(Subcommand)]
pub enum MessageAction {
    /// thread に新しい message を投稿する
    Post {
        /// thread ID
        #[arg(long)]
        thread: String,
        /// message の role（user, assistant, system, tool）
        #[arg(long, default_value = "user")]
        role: String,
        /// message の内容（省略時は stdin から読み取り）
        #[arg(long)]
        content: Option<String>,
        /// session ID
        #[arg(long)]
        session: Option<String>,
        /// 送信者名
        #[arg(long)]
        sender: Option<String>,
        /// 親 message の ID
        #[arg(long)]
        parent: Option<String>,
        /// JSON 文字列形式のメタデータ
        #[arg(long)]
        metadata: Option<String>,
    },
    /// thread の message を読み取る
    Read {
        /// thread ID
        #[arg(long)]
        thread: String,
        /// 返す message の最大件数
        #[arg(long)]
        limit: Option<usize>,
        /// この日時より前の message のみ（ISO 8601）
        #[arg(long)]
        before: Option<String>,
        /// この日時より後の message のみ（ISO 8601）
        #[arg(long)]
        after: Option<String>,
        /// 出力形式（text, json）
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// message を検索する
    Search {
        /// 検索クエリ
        query: String,
        /// 特定の thread に検索を限定
        #[arg(long)]
        thread: Option<String>,
        /// 出力形式（text, json, markdown）
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// message の内容を更新する
    Update {
        /// message ID（短い prefix でも可）
        id: String,
        /// 新しい内容
        #[arg(long)]
        content: String,
    },
}

#[derive(Subcommand)]
pub enum ThreadAction {
    /// 新しい thread を作成する
    Create {
        /// thread のタイトル
        title: String,
    },
    /// 全 thread を一覧表示する
    List {
        /// 出力形式（text, json）
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// thread とその message を削除する
    Delete {
        /// thread ID
        id: String,
    },
    /// URL から会話を取得して保存する
    Fetch {
        /// 取得元 URL
        url: String,
        /// thread のタイトル（省略時は URL を使用）
        #[arg(long)]
        title: Option<String>,
        /// 取得コンテンツの送信者名
        #[arg(long)]
        sender: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum HookAction {
    /// stdin から Claude Code hook イベントを取り込む
    Ingest {
        /// thread ID の上書き（省略時は stdin JSON の session_id を使用）
        #[arg(long)]
        thread: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CleanupAction {
    /// N 日より古い message を削除する
    Age {
        /// 日数
        days: i64,
        /// DB バックアップをスキップする
        #[arg(long)]
        no_backup: bool,
    },
    /// thread とその全 message を削除する
    Thread {
        /// thread ID
        id: String,
        /// DB バックアップをスキップする
        #[arg(long)]
        no_backup: bool,
    },
    /// session の全 message を削除する
    Session {
        /// session ID
        id: String,
        /// DB バックアップをスキップする
        #[arg(long)]
        no_backup: bool,
    },
}

#[derive(Subcommand)]
pub enum SetupAction {
    /// Claude Code 用の hook 設定を生成する
    Hooks {
        /// 生成した設定を .claude/settings.json に適用する
        #[arg(long)]
        apply: bool,
    },
    /// Claude Code 用の aiboard skill ファイルを生成する
    Skill {
        /// 生成した skill を .claude/skills/ に適用する
        #[arg(long)]
        apply: bool,
    },
}
