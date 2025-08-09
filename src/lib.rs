// lib.rs

#![deny(clippy::all)]
use std::collections::HashMap;

use napi::bindgen_prelude::Function;
use napi_derive::napi;
use tao::{
  event::Event,
  event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{menu::MenuEvent, TrayIconEvent};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::menu_parse::JsMenu;
mod icon_parse;
mod menu_parse;
enum UserEvent {
  #[allow(dead_code)]
  TrayIconEvent(tray_icon::TrayIconEvent),
  MenuEvent(tray_icon::menu::MenuEvent),
}
#[napi]
pub struct SystemTray {
  engine: TrayIcon,
  callback_map: HashMap<String, Function<'static, (), ()>>,
}

#[napi]
impl SystemTray {
  #[napi(constructor)]
  pub fn new(id: Option<String>) -> Self {
    let mut tray_builder = TrayIconBuilder::new();
    if let Some(id) = id {
      tray_builder = tray_builder.with_id(id);
    }
    let engine = tray_builder.build().unwrap();
    SystemTray {
      engine: engine,
      callback_map: HashMap::new(),
    }
  }

  #[napi]
  pub fn id(&self) -> String {
    self.engine.id().0.clone()
  }

  #[napi]
  pub fn set_icon(&self, icon: String) {
    let icon = icon_parse::parse_icon_from_string(icon);
    self
      .engine
      .set_icon(Some(icon))
      .expect("Failed to set icon");
  }

  #[napi]
  pub fn set_tooltip(&self, tooltip: String) {
    self
      .engine
      .set_tooltip(Some(tooltip))
      .expect("Failed to set tooltip");
  }

  #[napi]
  pub fn set_title(&self, title: String) {
    self.engine.set_title(Some(title));
  }

  #[napi]
  pub fn set_visible(&self, visible: bool) {
    self
      .engine
      .set_visible(visible)
      .expect("Failed to set visible");
  }

  #[napi]
  pub fn set_menu(&mut self, js_menu: JsMenu) {
    self.callback_map.clear();
    self
      .callback_map
      .extend(menu_parse::get_callback_map_from_menu(&js_menu));
    let menu = menu_parse::js_menu_to_tray_menu(&js_menu);
    self.engine.set_menu(Some(Box::new(menu)));
  }
  #[napi]
  pub fn listen(&self) {
    let callback_map = self.callback_map.clone();
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
      let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }));

    // set a menu event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
      let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));
    let _menu_channel = MenuEvent::receiver();
    let _tray_channel = TrayIconEvent::receiver();
    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Wait;
      match event {
        Event::NewEvents(tao::event::StartCause::Init) => {
          // We have to request a redraw here to have the icon actually show up.
          // Tao only exposes a redraw method on the Window so we use core-foundation directly.
          #[cfg(target_os = "macos")]
          unsafe {
            use objc2_core_foundation::{CFRunLoopGetMain, CFRunLoopWakeUp};

            let rl = CFRunLoopGetMain().unwrap();
            CFRunLoopWakeUp(&rl);
          }
        }

        Event::UserEvent(UserEvent::MenuEvent(event)) => {
          if let Some(cb) = callback_map.get(&event.id().0) {
            cb.call(()).unwrap();
          }
        }
        _ => {}
      }
    });
  }
}
