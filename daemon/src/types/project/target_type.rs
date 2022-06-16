use super::*;

#[derive(Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum ProjectTargetType {
    #[serde(rename = "application")]
    Application,
    #[serde(rename = "application.on-demand-install-capable")]
    ApplicationOnDemandInstallCapable,
    #[serde(rename = "application.messages")]
    ApplicationMessages,
    #[serde(rename = "application.watchapp")]
    ApplicationWatchApp,
    #[serde(rename = "application.watchapp2")]
    ApplicationWatchApp2,
    #[serde(rename = "app-extension")]
    AppExtension,
    #[serde(rename = "app-extension.intents-service")]
    AppExtensionIntentsService,
    #[serde(rename = "app-extension.messages")]
    AppExtensionMessages,
    #[serde(rename = "app-extension.messages-sticker-pack")]
    AppExtensionMessagesStickerPack,
    #[serde(rename = "bundle")]
    Bundle,
    #[serde(rename = "bundle.ocunit-test")]
    BundleOCunitTest,
    #[serde(rename = "bundle.ui-testing")]
    BundleUITesting,
    #[serde(rename = "bundle.unit-test")]
    BundleUITest,
    #[serde(rename = "framework")]
    Framework,
    #[serde(rename = "instruments-package")]
    InstrumentsPackage,
    #[serde(rename = "library.dynamic")]
    LibraryDynamic,
    #[serde(rename = "library.static")]
    LibraryStatic,
    #[serde(rename = "framework.static")]
    FrameworkStatic,
    #[serde(rename = "tool")]
    Tool,
    #[serde(rename = "tv-app-extension")]
    TvAppExtension,
    #[serde(rename = "watchapp2-container")]
    WatchApp2Container,
    #[serde(rename = "watchkit-extension")]
    WatchKitExtension,
    #[serde(rename = "watchkit2-extension")]
    WatchKit2Extension,
    #[serde(rename = "xcode-extension")]
    XcodeExtension,
    #[serde(rename = "driver-extension")]
    DriverExtension,
    #[serde(rename = "system-extension")]
    SystemExtension,
    #[serde(rename = "xpc-service")]
    XPCService,
}

impl Default for ProjectTargetType {
    fn default() -> Self {
        Self::Application
    }
}

impl ProjectTargetType {
    #[must_use]
    pub fn is_application(&self) -> bool {
        matches!(self, Self::Application)
    }
    #[must_use]
    pub fn is_application_on_demand_install_capable(&self) -> bool {
        matches!(self, Self::ApplicationOnDemandInstallCapable)
    }
    #[must_use]
    pub fn is_application_messages(&self) -> bool {
        matches!(self, Self::ApplicationMessages)
    }
    #[must_use]
    pub fn is_application_watch_app(&self) -> bool {
        matches!(self, Self::ApplicationWatchApp)
    }
    #[must_use]
    pub fn is_application_watch_app2(&self) -> bool {
        matches!(self, Self::ApplicationWatchApp2)
    }
    #[must_use]
    pub fn is_app_extension(&self) -> bool {
        matches!(self, Self::AppExtension)
    }
    #[must_use]
    pub fn is_app_extension_intents_service(&self) -> bool {
        matches!(self, Self::AppExtensionIntentsService)
    }
    #[must_use]
    pub fn is_app_extension_messages(&self) -> bool {
        matches!(self, Self::AppExtensionMessages)
    }
    #[must_use]
    pub fn is_app_extension_messages_sticker_pack(&self) -> bool {
        matches!(self, Self::AppExtensionMessagesStickerPack)
    }
    #[must_use]
    pub fn is_bundle(&self) -> bool {
        matches!(self, Self::Bundle)
    }
    #[must_use]
    pub fn is_bundle_ocunit_test(&self) -> bool {
        matches!(self, Self::BundleOCunitTest)
    }
    #[must_use]
    pub fn is_bundle_uitesting(&self) -> bool {
        matches!(self, Self::BundleUITesting)
    }
    #[must_use]
    pub fn is_bundle_uitest(&self) -> bool {
        matches!(self, Self::BundleUITest)
    }
    #[must_use]
    pub fn is_framework(&self) -> bool {
        matches!(self, Self::Framework)
    }
    #[must_use]
    pub fn is_instruments_package(&self) -> bool {
        matches!(self, Self::InstrumentsPackage)
    }
    #[must_use]
    pub fn is_library_dynamic(&self) -> bool {
        matches!(self, Self::LibraryDynamic)
    }
    #[must_use]
    pub fn is_library_static(&self) -> bool {
        matches!(self, Self::LibraryStatic)
    }
    #[must_use]
    pub fn is_framework_static(&self) -> bool {
        matches!(self, Self::FrameworkStatic)
    }
    #[must_use]
    pub fn is_tv_app_extension(&self) -> bool {
        matches!(self, Self::TvAppExtension)
    }
    #[must_use]
    pub fn is_watch_app2_container(&self) -> bool {
        matches!(self, Self::WatchApp2Container)
    }
    #[must_use]
    pub fn is_watch_kit_extension(&self) -> bool {
        matches!(self, Self::WatchKitExtension)
    }
    #[must_use]
    pub fn is_watch_kit2_extension(&self) -> bool {
        matches!(self, Self::WatchKit2Extension)
    }
    #[must_use]
    pub fn is_xcode_extension(&self) -> bool {
        matches!(self, Self::XcodeExtension)
    }
    #[must_use]
    pub fn is_driver_extension(&self) -> bool {
        matches!(self, Self::DriverExtension)
    }
    #[must_use]
    pub fn is_system_extension(&self) -> bool {
        matches!(self, Self::SystemExtension)
    }
    #[must_use]
    pub fn is_xpcservice(&self) -> bool {
        matches!(self, Self::XPCService)
    }
}
