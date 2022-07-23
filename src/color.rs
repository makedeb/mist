pub use colored::Colorize;
use colored::CustomColor;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref UBUNTU_ORANGE: CustomColor = CustomColor::new(255, 175, 0);
    pub static ref UBUNTU_PURPLE: CustomColor = CustomColor::new(95, 95, 255);
}
