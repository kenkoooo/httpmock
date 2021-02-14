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
pub struct StringValue {
    value: String,
    encoding: Option<Encoding>,
}

impl<T: Into<String>> From<T> for StringValue {
    fn from(value: T) -> Self {
        StringValue {
            value: value.into(),
            encoding: None,
        }
    }
}

impl Into<StringValue> for &StringValue {
    fn into(self) -> StringValue {
        StringValue {
            value: self.value.to_string(),
            encoding: self.encoding.clone(),
        }
    }
}

// ************************************************************************************
// The following methods provide url encoding for StringValue instances.
// ************************************************************************************
pub trait URLEncodedExtension {
    fn url_encoded(&self) -> StringValue;
}

impl<T: ToString> URLEncodedExtension for T {
    fn url_encoded(&self) -> StringValue {
        StringValue {
            value: self.to_string(),
            encoding: Some(Encoding::URL),
        }
    }
}
