#[derive(Clone)]
enum Encoding {
    URL,
    Base64,
}

/// An httpmock specific representation of a string value with optional metadata (e.g. encoding).
/// httpmock provides automatic conversions to this type for all values that can be converted into
/// a string (i.e. implement trait Into<String>).
///
/// **Attention**: The end user must not use this type directly! Instead, use normal String, &str,
/// or anything else that can be converted into a String. httpmock already comes with automatic
/// conversions from string like types to StringValue on all methods that require it!
pub struct MaybeEncoded<T> {
    value: T,
    encoding: Option<Encoding>,
}

impl<T: Into<String>> From<T> for MaybeEncoded<String> {
    fn from(value: T) -> Self {
        MaybeEncoded {
            value: value.into(),
            encoding: None,
        }
    }
}

impl Into<MaybeEncoded<String>> for &MaybeEncoded<String> {
    fn into(self) -> MaybeEncoded<String> {
        MaybeEncoded {
            value: self.value.to_string(),
            encoding: self.encoding.clone(),
        }
    }
}

// ************************************************************************************
// The following methods provide url encoding for StringValue instances.
// ************************************************************************************
pub trait URLEncodedExtension<T> {
    fn url_encoded(&self) -> MaybeEncoded<T>;
}

impl<T: ToString> URLEncodedExtension<String> for T {
    fn url_encoded(&self) -> MaybeEncoded<String> {
        url_encoded(self.to_string())
    }
}

pub fn url_encoded<S, T: Into<S>>(value: T) ->  MaybeEncoded<S> {
    MaybeEncoded {
        value: value.into(),
        encoding: Some(Encoding::URL),
    }
}