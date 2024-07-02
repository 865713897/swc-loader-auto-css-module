use swc_core::ecma::{
    ast::{ ImportDecl, Module, Program },
    visit::{ as_folder, FoldWith, VisitMut, VisitMutWith },
};
use swc_core::plugin::{ plugin_transform, proxies::TransformPluginProgramMetadata };
use serde::Deserialize;

const DEFAULT_CSS_SUFFIX: [&str; 4] = [".css", ".less", ".scss", ".sass"];
const DEFAULT_FILE_SUFFIX: &str = "css_modules";

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum CssSuffixConfig {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Debug)]
pub struct PluginConfig {
    css_suffix: Option<CssSuffixConfig>,
    file_suffix: Option<String>,
}

#[derive(Debug)]
pub struct TransformVisitor {
    css_suffix: Vec<String>,
    file_suffix: String,
}

impl TransformVisitor {
    pub fn new(css_suffix: Vec<String>, file_suffix: String) -> Self {
        Self { css_suffix, file_suffix }
    }
}

impl VisitMut for TransformVisitor {
    fn visit_mut_module(&mut self, n: &mut Module) {
        n.visit_mut_children_with(self);
    }

    fn visit_mut_import_decl(&mut self, n: &mut ImportDecl) {
        if !n.specifiers.is_empty() {
            if let Some(extension) = n.src.value.rsplit('.').next() {
                let extension_with_dot = format!(".{}", extension);
                if self.css_suffix.contains(&extension_with_dot) {
                    let new_value = format!("{}?{}", n.src.value, self.file_suffix);
                    n.src.value = new_value.clone().into();
                    n.src.raw = Some(format!("\"{}\"", new_value).into());
                }
            }
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, data: TransformPluginProgramMetadata) -> Program {
    let raw_config = data.get_transform_plugin_config().expect("failed to get plugin config");

    let config: PluginConfig = serde_json
        ::from_str(&raw_config)
        .unwrap_or_else(|_| { PluginConfig { css_suffix: None, file_suffix: None } });

    let css_suffix = match config.css_suffix {
        Some(CssSuffixConfig::Single(suffix)) => vec![suffix],
        Some(CssSuffixConfig::Multiple(suffixes)) => suffixes,
        None =>
            DEFAULT_CSS_SUFFIX.iter()
                .map(|&s| s.to_string())
                .collect(),
    };

    let file_suffix = config.file_suffix.unwrap_or_else(|| DEFAULT_FILE_SUFFIX.to_string());

    program.fold_with(&mut as_folder(TransformVisitor::new(css_suffix, file_suffix)))
}
