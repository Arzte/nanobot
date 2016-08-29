use hyper::Client;
use serde_json;
use ::error::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
pub struct LocationLocation {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LocationGeometry {
    pub location: LocationLocation,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AddressComponent {
    pub long_name: String,
    pub short_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LocationResult {
    pub address_components: Vec<AddressComponent>,
    pub geometry: LocationGeometry,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LocationData {
    pub results: Vec<LocationResult>,
}

pub fn get_address<S: Into<String>>(address: S) -> Result<LocationData> {
    let address = address.into();

    let url = format!("https://maps.googleapis.com/maps/api/geocode/json?address={}",
                      address.replace(' ', "+"));

    let response = match Client::new().get(&url).send() {
        Ok(response) => response,
        Err(why) => {
            warn!("[google-maps] err getting loc {}: {:?}", address, why);

            return Err(Error::Hyper(why));
        },
    };

    let location_data: LocationData = match serde_json::from_reader(response) {
        Ok(location_data) => location_data,
        Err(why) => {
            warn!("[google-maps] err decoding loc {}: {:?}", address, why);

            return Err(Error::Json(why));
        },
    };

    Ok(location_data)
}
