mod common;

use acia::explanation::CommandExplanationFormatter;
use acia::llm::CommandCandidate;
use acia::llm::SafetyLevel;
use acia::llm::response::{CommandComponent, CommandExplanation};

#[test]
fn test_display_mock_command() {
    // Create a mock command for display testing
    let mock_command = CommandCandidate {
        command: "grep -r \"apple\" .".to_string(),
        description: "カレントディレクトリ内のすべてのファイルで「apple」という単語を検索します"
            .to_string(),
        safety_level: SafetyLevel::Safe,
        explanation: CommandExplanation {
            purpose: "ファイル内のappleという単語を検索します".to_string(),
            components: vec![
                CommandComponent {
                    part: "grep".to_string(),
                    explanation: "ファイル内のテキストパターンを検索するコマンド".to_string(),
                },
                CommandComponent {
                    part: "-r".to_string(),
                    explanation: "再帰的にサブディレクトリも含めて検索する（recursive）"
                        .to_string(),
                },
                CommandComponent {
                    part: "\"apple\"".to_string(),
                    explanation: "検索する単語（パターン）".to_string(),
                },
                CommandComponent {
                    part: ".".to_string(),
                    explanation: "カレントディレクトリを検索対象とする".to_string(),
                },
            ],
        },
    };

    // Create formatter with default terminal width
    let formatter = CommandExplanationFormatter::default()
        .with_terminal_width(80)
        .with_related_commands(true);

    // Format and print the explanation
    let formatted = formatter.format(&mock_command);

    // Print the formatted output to console for manual verification
    println!("\n----- MOCK COMMAND EXPLANATION -----");
    println!("{}", formatted);
    println!("----- END MOCK EXPLANATION -----\n");

    // Verify key components are included
    assert!(formatted.contains("COMMAND:"));
    assert!(formatted.contains("grep -r \"apple\" ."));
    assert!(formatted.contains("PURPOSE:"));
    assert!(formatted.contains("OPTIONS:"));
    assert!(formatted.contains("grep"));
    assert!(formatted.contains("-r"));
    assert!(formatted.contains("\"apple\""));
    assert!(formatted.contains("RELATED:"));
}
