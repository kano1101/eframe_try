use eframe;
use eframe::egui;
use eframe::egui::FontData;
use eframe::egui::FontDefinitions;
use eframe::egui::FontFamily;
use eframe::egui::Ui;

// use sqlx::prelude::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    tag: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "テスト World!".to_owned(),
            value: 2.7,
            tag: "123".to_string(),
        }
    }
}

use serde::Deserialize;
#[derive(Clone, Deserialize)]
struct Ip {
    origin: String,
}
struct SingletonImpl {
    ip: Ip,
    dirty: bool,
}
impl Default for SingletonImpl {
    fn default() -> SingletonImpl {
        SingletonImpl {
            ip: Ip {
                origin: "".to_string(),
            },
            dirty: true,
        }
    }
}
impl SingletonImpl {
    async fn fetch_ip(&mut self) -> anyhow::Result<()> {
        let response = reqwest::get("http://httpbin.org/ip")
            .await?
            .json::<Ip>()
            .await?;
        self.ip = Ip {
            origin: response.origin,
        };
        Ok(())
    }
    fn run<F, R>(&self, task: F, suffix: String) -> anyhow::Result<String>
    where
        F: std::future::Future<Output = anyhow::Result<R>> + 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(task)?;
        }
        #[cfg(target_arch = "wasm32")]
        {
            SingletonImpl::dirty();
            wasm_bindgen_futures::spawn_local(async move {
                task.await;
                SingletonImpl::clean();
            });
        }
        let mut a: String = SingletonImpl::as_ref().ip.origin.clone();
        a.push_str(suffix.as_str());
        Ok(a)
    }
}
trait ISingleton<T>: 'static {
    fn init();
    fn can_start() -> bool;
    fn dirty();
    fn clean();
    fn is_dirty() -> bool;
    fn as_ref() -> &'static T;
    fn as_mut() -> &'static mut T;
}
static mut ITEM: Option<Box<SingletonImpl>> = None;
impl ISingleton<SingletonImpl> for SingletonImpl {
    fn init() {
        if unsafe { ITEM.as_ref().is_none() } {
            unsafe {
                ITEM = Some(Box::new(SingletonImpl::default()));
            }
        }
    }
    fn can_start() -> bool {
        unsafe { ITEM.is_some() }
    }
    fn dirty() {
        SingletonImpl::init();
        SingletonImpl::as_mut().dirty = true;
    }
    fn clean() {
        SingletonImpl::init();
        SingletonImpl::as_mut().dirty = false;
    }
    fn is_dirty() -> bool {
        SingletonImpl::init();
        SingletonImpl::as_ref().dirty
    }
    fn as_ref() -> &'static SingletonImpl {
        SingletonImpl::init();
        unsafe { ITEM.as_ref().unwrap() }
    }
    fn as_mut() -> &'static mut SingletonImpl {
        SingletonImpl::init();
        unsafe { ITEM.as_mut().unwrap() }
    }
}

fn perform(num: impl Into<String>) -> String {
    let task = SingletonImpl::as_mut().fetch_ip();
    let body = SingletonImpl::as_ref().run(task, num.into()).unwrap();
    body
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> TemplateApp {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_string(),
            FontData::from_static(include_bytes!("../fonts/SourceHanSansJP-Regular.otf")),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "my_font".to_string());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "my_font".to_string());
        cc.egui_ctx.set_fonts(fonts);

        cc.egui_ctx.set_visuals(egui::Visuals::default());

        perform("12345");

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        } else {
            return Default::default();
        }
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self { label, value, tag } = self;

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.button("終了").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            let body = perform(self.tag.clone());
            ctx.request_repaint_after(std::time::Duration::from_millis(500u64));

            ui.heading(body);
            ui.horizontal(|ui| {
                ui.label("何か書いてください: ".to_string());
                if ui.text_edit_singleline(label).clicked() {
                    self.tag = "アイウ".to_string();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
