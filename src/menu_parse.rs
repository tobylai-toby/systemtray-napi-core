// src/menu_parse.rs

use std::collections::HashMap;

use napi::bindgen_prelude::Function;
use napi_derive::napi;
use rand::{rng, Rng};
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

#[napi(object, js_name = "MenuItem")]
pub struct JsMenuItem {
  pub id: Option<String>,
  pub label: Option<String>,
  #[napi(ts_type = "'normal' | 'separator' | 'submenu'")]
  pub r#type: Option<String>,
  pub enabled: Option<bool>,
  pub submenu: Option<Vec<JsMenuItem>>,
  pub callback: Option<Function<'static, (), ()>>,
}

#[napi(object, js_name = "Menu")]
pub struct JsMenu {
  pub items: Vec<JsMenuItem>,
}
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
fn random_id() -> String {
  (0..8)
    .map(|_| {
      let idx = rng().random_range(0..CHARSET.len());
      CHARSET[idx] as char
    })
    .collect()
}

fn to_native_item(item: &JsMenuItem) -> Result<Box<dyn tray_icon::menu::IsMenuItem>, String> {
  let enabled = item.enabled.unwrap_or(true);

  match item.r#type.as_deref() {
    Some("separator") => {
      let sep = PredefinedMenuItem::separator();
      Ok(Box::new(sep))
    }

    Some("submenu") => {
      let submenu;
      if let Some(id) = &item.id {
        submenu = Submenu::with_id(id, item.label.clone().unwrap_or_default(), enabled);
      } else {
        submenu = Submenu::with_id(random_id(), item.label.clone().unwrap_or_default(), enabled);
      }
      if let Some(ref subitems) = item.submenu {
        let mut items = Vec::new();
        for sub in subitems {
          let native = to_native_item(sub)?;
          items.push(native);
        }
        let refs: Vec<&dyn tray_icon::menu::IsMenuItem> =
          items.iter().map(|i| i.as_ref()).collect();
        submenu.append_items(&refs).map_err(|e| e.to_string())?;
      }

      Ok(Box::new(submenu))
    }

    _ => {
      let menu_item;
      if let Some(id) = &item.id {
        menu_item = MenuItem::with_id(id, item.label.clone().unwrap_or_default(), enabled, None);
      } else {
        menu_item = MenuItem::with_id(
          random_id(),
          item.label.clone().unwrap_or_default(),
          enabled,
          None,
        );
      }
      Ok(Box::new(menu_item))
    }
  }
}

fn get_callback_map(item: &JsMenuItem) -> HashMap<String, Function<'static, (), ()>> {
  let mut map = HashMap::new();
  if let Some(cb) = item.callback {
    map.insert(item.id.clone().unwrap_or_default(), cb);
  }
  if let Some("submenu") = item.r#type.as_deref() {
    if let Some(submenu) = item.submenu.as_ref() {
      for subitem in submenu {
        map.extend(get_callback_map(subitem));
      }
    }
  }
  map
}
pub fn get_callback_map_from_menu(menu: &JsMenu) -> HashMap<String, Function<'static, (), ()>> {
  let mut map = HashMap::new();
  for item in &menu.items {
    map.extend(get_callback_map(item));
  }
  map
}

pub fn js_menu_to_tray_menu(js_menu: &JsMenu) -> Menu {
  let menu = Menu::new();

  for item in &js_menu.items {
    let native_item = to_native_item(item).expect("Failed to convert JS menu item to native"); // 调用递归函数
    menu
      .append(native_item.as_ref())
      .expect("Failed to append menu item");
  }

  menu
}
