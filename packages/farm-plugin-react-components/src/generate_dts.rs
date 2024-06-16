use crate::find_local_components::{ComponentInfo, ExportType};
use farmfe_toolkit::regex::Regex;
use farmfe_utils::relative;
use std::{
  fs::File,
  io::{BufWriter, Write},
  path::Path,
};

pub struct GenerateDtsOption<'a> {
  pub components: &'a Vec<&'a ComponentInfo>,
  pub resolvers_components: &'a Vec<&'a ComponentInfo>,
  pub root_path: String,
  pub filename: String,
  pub local: bool,
}

fn remove_tsx_jsx_suffix(s: &str) -> String {
  let re = Regex::new(r"\.[tj]sx$").unwrap();
  re.replace(s, "").into_owned()
}

pub fn stringify_resolver(item: &ComponentInfo) -> String {
  let is_export_component = match item.export_type {
    ExportType::Named => true,
    _ => false,
  };
  let mut target = "default";
  if is_export_component {
    target = &item.original_name;
  }
  format!(
    "\tconst {}: typeof import('{}')['{}']\n",
    item.name, item.path, target
  )
}

pub fn stringify_component(root_path: &str, item: &ComponentInfo) -> String {
  let related = format!("./{}", relative(root_path, &item.path));
  let import_path = remove_tsx_jsx_suffix(&related);
  let is_export_component = match item.export_type {
    ExportType::Named => true,
    _ => false,
  };
  let mut target = "default";
  if is_export_component {
    target = &item.name;
  }
  format!(
    "\tconst {}: typeof import('{}')['{}']\n",
    item.name, import_path, target
  )
}

pub fn generate_dts(option: GenerateDtsOption) {
  let mut code =
    "/* generated by farm_plugin_react_components */\nexport {} \ndeclare global {\n".to_string();
  let dts_output = Path::new(&option.root_path).join(Path::new(&option.filename));
  if option.local {
    code.push_str(
      &option
        .components
        .iter()
        .map(|&s| stringify_component(&option.root_path, s))
        .collect::<Vec<_>>()
        .join(""),
    );
  }
  code.push_str(
    &option
      .resolvers_components
      .iter()
      .map(|&item| stringify_resolver(item))
      .collect::<Vec<_>>()
      .join(""),
  );

  code.push_str("}");
  let file = File::create(dts_output).unwrap();
  let mut writer = BufWriter::new(file);
  writeln!(writer, "{}", code).unwrap();
  writer.flush().unwrap();
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    find_local_components::find_local_components,
    resolvers::{get_resolvers_result, ResolverOption},
  };
  use std::env;
  #[test]
  fn test_generate_dts() {
    let current_dir = env::current_dir().unwrap();
    let binding = current_dir.join("playground");
    let root_path = binding.to_str().unwrap();
    let components = find_local_components(root_path);
    let resolvers = [ResolverOption {
      module: "antd".to_string(),
      export_type: Some(ExportType::Named),
      style: Some(false),
      exclude: None,
      include: None,
      prefix: Some("Ant".to_string()),
    }];
    let resolvers_components = get_resolvers_result(&root_path, resolvers.to_vec());
    let generate_dts_option = GenerateDtsOption {
      components: &components.iter().collect::<Vec<_>>(),
      resolvers_components: &resolvers_components.iter().collect::<Vec<_>>(),
      root_path: root_path.to_string(),
      filename: "components.d.ts".to_string(),
      local: true,
    };
    generate_dts(generate_dts_option);
    assert!(!components.is_empty(), "Components should not be empty");
  }
}
