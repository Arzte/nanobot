use hyper::Client;
use serde_json::{self, Value};
use ::error::{Error, Result};
use ::utils::{decode_array, into_map, into_string, opt, remove};

#[derive(Clone, Debug)]
pub struct LocationLocation {
    pub lat: f64,
    pub lng: f64,
}

impl LocationLocation {
    fn decode(value: Value) -> Result<Self> {
        let mut map = try!(into_map(value));

        Ok(LocationLocation {
            lat: reqf!(try!(remove(&mut map, "lat")).as_f64()),
            lng: reqf!(try!(remove(&mut map, "lng")).as_f64()),
        })
    }
}

#[derive(Clone, Debug)]
pub struct LocationGeometry {
    pub location: LocationLocation,
}

impl LocationGeometry {
    fn decode(value: Value) -> Result<Self> {
        let mut map = try!(into_map(value));

        Ok(LocationGeometry {
            location: try!(remove(&mut map, "location")
                .and_then(LocationLocation::decode)),
        })
    }
}

#[derive(Clone, Debug)]
pub struct AddressComponent {
    pub long_name: String,
    pub short_name: String,
}

impl AddressComponent {
    fn decode(value: Value) -> Result<Self> {
        let mut map = try!(into_map(value));

        Ok(AddressComponent {
            long_name: try!(remove(&mut map, "long_name")
                .and_then(into_string)),
            short_name: try!(remove(&mut map, "short_name")
                .and_then(into_string)),
        })
    }
}

#[derive(Clone, Debug)]
pub struct LocationResult {
    pub address_components: Vec<AddressComponent>,
    pub geometry: LocationGeometry,
}

impl LocationResult {
    fn decode(value: Value) -> Result<Self> {
        let mut map = try!(into_map(value));

        Ok(LocationResult {
            address_components: try!(remove(&mut map, "address_components")
                .and_then(|v| decode_array(v, AddressComponent::decode))),
            geometry: try!(remove(&mut map, "geometry")
                .and_then(LocationGeometry::decode)),
        })
    }
}

#[derive(Clone, Debug)]
pub struct LocationData {
    pub results: Vec<LocationResult>,
}

impl LocationData {
    fn decode(value: Value) -> Result<Self> {
        let mut map = try!(into_map(value));

        Ok(LocationData {
            results: try!(opt(&mut map, "results", |v| {
                decode_array(v, LocationResult::decode)
            })).unwrap_or(vec![]),
        })
    }
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

    match LocationData::decode(try!(serde_json::from_reader(response))) {
        Ok(data) => Ok(data),
        Err(why) => {
            warn!("[google-maps] err decoding loc {}: {:?}", address, why);

            Err(why)
        },
    }
}
