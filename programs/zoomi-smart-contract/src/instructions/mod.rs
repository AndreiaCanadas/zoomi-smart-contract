pub mod register_rider;
pub mod register_scooter;
pub mod start_rental;
pub mod update_scooter_location;
pub mod set_scooter_status;         // Scooter Status updated by Shopkeeper
pub mod update_scooter_status;      // Scooter Status updated by Scooter Device
pub mod end_rental;
pub mod extend_rental_period;

pub use register_rider::*;
pub use register_scooter::*;
pub use start_rental::*;
pub use update_scooter_location::*;
pub use set_scooter_status::*;
pub use update_scooter_status::*;
pub use end_rental::*;
pub use extend_rental_period::*;