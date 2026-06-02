use crate::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_TIMEOUT_MS: u64 = 120_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRuntimeState {
    pub enabled: bool,
    pub runtime_version: String,
    pub headless_default: bool,
    pub installed: bool,
    pub ready: bool,
    pub installing: bool,
    pub last_error: String,
    pub node_version: String,
    pub playwright_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationSettings {
    pub enabled: bool,
    pub headless_default: bool,
}

impl Default for AutomationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            headless_default: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationSelfCheck {
    pub ok: bool,
    pub node_source: String,
    pub node_version: String,
    pub playwright_version: String,
    pub runner_path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationScript {
    pub script_id: String,
    pub name: String,
    pub description: String,
    pub entry_file: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationScriptInput {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRun {
    pub run_id: String,
    pub script_id: String,
    pub script_name: String,
    pub profile_id: String,
    pub profile_name: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: i64,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub result_json: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunInput {
    pub script_id: String,
    pub profile_id: String,
    pub params: Option<Value>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunEvent {
    pub run_id: String,
    pub status: String,
    pub message: String,
    pub run: Option<AutomationRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunLogEvent {
    pub run_id: String,
    pub stream: String,
    pub line: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRuntimeProgressEvent {
    pub phase: String,
    pub message: String,
    pub progress: i32,
}

#[derive(Debug, Clone)]
pub struct AutomationQueuedTask {
    pub run_id: String,
    pub input: AutomationRunInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInstallManifest {
    pub version: String,
    pub package_name: String,
    pub download_url: String,
    pub installed_at: String,
}

#[derive(Debug, Clone)]
pub struct NodePackage {
    pub version: String,
    pub package_name: String,
    pub url: String,
    pub version_dir: PathBuf,
    pub archive_path: PathBuf,
    pub executable_path: PathBuf,
    pub npm_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AutomationPaths {
    pub root: PathBuf,
    pub settings: PathBuf,
    pub runtime: PathBuf,
    pub node_root: PathBuf,
    pub node_downloads: PathBuf,
    pub node_versions: PathBuf,
    pub node_current_version: PathBuf,
    pub runner: PathBuf,
    pub scripts: PathBuf,
    pub runs: PathBuf,
}

impl AutomationPaths {
    pub fn new(app_root: &Path) -> Self {
        let root = app_root.join("data").join("automation");
        let runtime = root.join("runtime");
        let node_root = runtime.join("node");
        Self {
            settings: root.join("settings.json"),
            runtime: runtime.clone(),
            node_downloads: node_root.join("downloads"),
            node_versions: node_root.join("versions"),
            node_current_version: node_root.join("current_version.json"),
            node_root,
            runner: root.join("runner").join("runner.cjs"),
            scripts: root.join("scripts"),
            runs: root.join("runs"),
            root,
        }
    }

    pub fn ensure(&self) -> Result<(), AppError> {
        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(&self.runtime)?;
        fs::create_dir_all(&self.node_root)?;
        fs::create_dir_all(&self.node_downloads)?;
        fs::create_dir_all(&self.node_versions)?;
        fs::create_dir_all(
            self.runner
                .parent()
                .ok_or_else(|| AppError::other("runner path has no parent"))?,
        )?;
        fs::create_dir_all(&self.scripts)?;
        fs::create_dir_all(&self.runs)?;
        Ok(())
    }
}

pub struct RunnerResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub result_json: String,
    pub error: String,
}

pub const RUNNER_CJS: &str = r#"
const fs = require('fs')
const { chromium } = require('playwright-core')

function readStdin() {
  return new Promise((resolve, reject) => {
    let raw = ''
    process.stdin.setEncoding('utf8')
    process.stdin.on('data', chunk => { raw += chunk })
    process.stdin.on('end', () => resolve(raw))
    process.stdin.on('error', reject)
  })
}

function log(message, data) {
  const item = { time: new Date().toISOString(), message, data }
  process.stderr.write(`[automation] ${JSON.stringify(item)}\n`)
}

function emitResult(payload) {
  process.stdout.write(`__YUBAI_AUTOMATION_RESULT__${JSON.stringify(payload)}\n`)
}

function loadUserModule(scriptPath) {
  if (scriptPath.endsWith('.ts')) {
    const ts = require('typescript')
    const source = fs.readFileSync(scriptPath, 'utf8')
    const compiled = ts.transpileModule(source, {
      compilerOptions: {
        module: ts.ModuleKind.CommonJS,
        target: ts.ScriptTarget.ES2022,
        esModuleInterop: true,
      },
      fileName: scriptPath,
    })
    const module = { exports: {} }
    const fn = new Function('require', 'module', 'exports', '__filename', '__dirname', compiled.outputText)
    fn(require, module, module.exports, scriptPath, require('path').dirname(scriptPath))
    return module.exports
  }
  return require(scriptPath)
}

async function main() {
  const raw = await readStdin()
  const input = JSON.parse(raw || '{}')
  if (input.healthCheck) {
    const pkg = require('playwright-core/package.json')
    emitResult({ ok: true, playwrightVersion: pkg.version })
    return
  }

  if (!input.scriptPath || !fs.existsSync(input.scriptPath)) {
    throw new Error('script entry file does not exist')
  }

  const userModule = loadUserModule(input.scriptPath)
  if (typeof userModule.run !== 'function') {
    throw new Error('script must export async function run(ctx)')
  }

  let browser
  try {
    browser = await chromium.connectOverCDP(`http://127.0.0.1:${input.debugPort}`)
    const context = browser.contexts()[0] || await browser.newContext()
    const page = context.pages()[0] || await context.newPage()
    const result = await userModule.run({
      browser,
      context,
      page,
      profileId: input.profileId,
      scriptId: input.scriptId,
      runId: input.runId,
      params: input.params || {},
      log,
    })
    emitResult({ ok: true, result: result === undefined ? null : result })
  } finally {
    if (browser) {
      await browser.close().catch(() => {})
    }
  }
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    emitResult({
      ok: false,
      error: error && error.message ? error.message : String(error),
      stack: error && error.stack ? error.stack : '',
    })
    process.exit(1)
  })
"#;
