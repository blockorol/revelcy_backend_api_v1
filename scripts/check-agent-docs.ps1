$ErrorActionPreference = "Stop"

$requiredFiles = @(
    "AGENTS.md",
    ".github/copilot-instructions.md",
    "docs/agents/README.md",
    "docs/agents/QUICK_START.md",
    "docs/agents/PROJECT_MAP.md",
    "docs/agents/ROUTE_HANDLERS.md",
    "docs/agents/ARCHITECTURE.md",
    "docs/agents/DOMAIN.md",
    "docs/agents/GLOSSARY.md",
    "docs/agents/INVARIANTS.md",
    "docs/agents/API_CONTRACTS.md",
    "docs/agents/DATABASE.md",
    "docs/agents/SOLANA.md",
    "docs/agents/AUTH_SECURITY.md",
    "docs/agents/ERROR_HANDLING.md",
    "docs/agents/DEPENDENCIES.md",
    "docs/agents/RISK_REGISTER.md",
    "docs/agents/LEGACY_NOTES.md",
    "docs/agents/MAPPING_GUIDE.md",
    "docs/agents/CODE_STYLE.md",
    "docs/agents/WORKFLOWS.md",
    "docs/agents/CHANGE_PROTOCOL.md",
    "docs/agents/TESTING.md",
    "docs/agents/FRONTEND_BACKEND_CONTRACT.md",
    "docs/agents/PR_REVIEW_CHECKLIST.md",
    "docs/agents/HUMAN_DOCS.md",
    "README.md",
    "docs/README.md",
    "docs/DEVELOPMENT.md",
    "docs/LOCAL_DATABASE.md",
    "docs/CONFIGURATION.md",
    "docs/API.md",
    "docs/API_EXAMPLES.md",
    "docs/API_CHANGELOG.md",
    "docs/ARCHITECTURE.md",
    "docs/DATABASE.md",
    "docs/SOLANA.md",
    "docs/AUTH.md",
    "docs/TESTING.md",
    "docs/DEPLOYMENT.md",
    "docs/RELEASE_PROCESS.md",
    "docs/TROUBLESHOOTING.md",
    "docs/DEPENDENCIES.md",
    "docs/GLOSSARY.md",
    "docs/OPEN_SOURCE_CHECKLIST.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "CHANGELOG.md",
    ".github/pull_request_template.md",
    ".github/workflows/ci.yml",
    ".github/workflows/docs.yml",
    ".github/workflows/security.yml",
    ".github/dependabot.yml",
    ".github/ISSUE_TEMPLATE/bug_report.md",
    ".github/ISSUE_TEMPLATE/feature_request.md",
    ".github/ISSUE_TEMPLATE/security_report.md",
    "deny.toml",
    "rustfmt.toml"
)

$requiredSkills = @(
    ".codex/skills/revelcy-update-agent-docs/SKILL.md",
    ".codex/skills/revelcy-update-agent-docs/agents/openai.yaml",
    ".codex/skills/revelcy-api-contract-review/SKILL.md",
    ".codex/skills/revelcy-api-contract-review/agents/openai.yaml",
    ".codex/skills/revelcy-db-change-review/SKILL.md",
    ".codex/skills/revelcy-db-change-review/agents/openai.yaml",
    ".codex/skills/revelcy-solana-change-review/SKILL.md",
    ".codex/skills/revelcy-solana-change-review/agents/openai.yaml",
    ".codex/skills/revelcy-pr-review/SKILL.md",
    ".codex/skills/revelcy-pr-review/agents/openai.yaml"
)

$missing = @()

foreach ($path in $requiredFiles + $requiredSkills) {
    if (-not (Test-Path -LiteralPath $path)) {
        $missing += $path
    }
}

if ($missing.Count -gt 0) {
    Write-Host "Missing agent files:"
    foreach ($path in $missing) {
        Write-Host " - $path"
    }
    exit 1
}

$todoMatches = Select-String -Path ".codex/skills/*/SKILL.md" -Pattern "\[TODO|TODO:" -ErrorAction SilentlyContinue
if ($todoMatches) {
    Write-Host "Skill TODO placeholders remain:"
    foreach ($match in $todoMatches) {
        Write-Host " - $($match.Path):$($match.LineNumber)"
    }
    exit 1
}

Write-Host "Agent documentation structure OK."
