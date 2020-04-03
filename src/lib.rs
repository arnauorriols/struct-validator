use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Display;
use std::iter::Extend;
use std::iter::FromIterator;

use derive_more::{From, IntoIterator, Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Display, Error, Debug, Default, IntoIterator, From, Serialize, Deserialize)]
#[display(fmt = "{}", "self.to_json_string()")]
pub struct StructValidator {
	pub errors: HashMap<String, String>,
}

impl StructValidator {
	pub fn new() -> Self {
		Self {
			errors: HashMap::new(),
		}
	}

	fn from_json_string<T>(str: T) -> Result<Self, serde_json::Error>
	where
		T: Into<String>,
	{
		let error_str = str.into();
		let line_info_index = error_str
			.rfind(" at line")
			.unwrap_or_else(|| error_str.len());
		let error_str = &error_str[..line_info_index];
		serde_json::from_str(error_str)
	}

	pub fn to_json_string(&self) -> String {
		serde_json::to_string(self)
			.unwrap_or_else(|e| format!("Error serializing StructValidator: {}", e))
	}

	pub fn insert<K, V>(&mut self, key: K, value: V)
	where
		K: Into<String>,
		V: Into<String>,
	{
		let error_str = value.into();
		let line_info_index = error_str
			.rfind(" at line")
			.unwrap_or_else(|| error_str.len());
		let error_str = error_str[..line_info_index].to_string();
		self.errors.insert(key.into(), error_str);
	}

	pub fn with<K, V>(mut self, key: K, value: V) -> Self
	where
		K: Into<String>,
		V: Into<String>,
	{
		self.insert(key, value);
		self
	}

	pub fn with_result<K, V, T>(self, key: K, result: Result<T, V>) -> Self
	where
		K: Into<String>,
		V: ToString,
	{
		if let Err(e) = result {
			self.with(key, e.to_string())
		} else {
			self
		}
	}

	pub fn contains<T>(&self, key: T) -> bool
	where
		T: Into<String>,
	{
		self.errors.contains_key(&key.into())
	}

	pub fn is_empty(&self) -> bool {
		self.errors.is_empty()
	}
}

impl TryFrom<serde_json::Error> for StructValidator {
	type Error = serde_json::Error;

	fn try_from(error: serde_json::Error) -> Result<Self, serde_json::Error> {
		Self::from_json_string(error.to_string())
	}
}

impl serde::de::Error for StructValidator {
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		let mut errors = Self::new();
		errors.insert("unknown".to_string(), msg.to_string());
		errors
	}
}

impl<'a, T: 'a> FromIterator<&'a Result<T, StructValidator>> for StructValidator {
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = &'a Result<T, StructValidator>>,
	{
		let mut errors = Self::new();
		for item in iter {
			if let Err(e) = item {
				errors.extend(e.clone());
			}
		}
		errors
	}
}

impl Extend<(String, String)> for StructValidator {
	fn extend<T>(&mut self, iter: T)
	where
		T: IntoIterator<Item = (String, String)>,
	{
		self.errors.extend(iter)
	}
}

impl<'a, T: 'a> Extend<&'a Result<T, StructValidator>> for StructValidator {
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = &'a Result<T, StructValidator>>,
	{
		let errors: StructValidator = iter.into_iter().collect();
		self.extend(errors);
	}
}

#[macro_export]
macro_rules! deserialize_struct {
($struct_name:ident, [$($field_name:ident),*], $explanation:literal) => {
	paste::item! {
		impl<'de> serde::Deserialize<'de> for $struct_name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				#[allow(non_camel_case_types)]
				enum Field {
					$($field_name),*,
					__ignore,
				};

				struct FieldVisitor;

				impl <'de> serde::de::Visitor<'de> for FieldVisitor {
					type Value = Field;
					fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
							write!(formatter, "a {}", stringify!($struct_name))
					}
					fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
					where E: serde::de::Error
					{
							match value {
								$(stringify!($field_name) => Ok(Field::$field_name)),*,
								_ => Ok(Field::__ignore)
							}
					}
				}

				impl <'de> serde::Deserialize<'de> for Field {
						#[inline]
						fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
						where D: serde::Deserializer<'de>
						{
								serde::Deserializer::deserialize_identifier(
									deserializer,
									FieldVisitor
								)
						}
				}

				struct [<$struct_name Visitor>];

				impl<'de> serde::de::Visitor<'de> for [<$struct_name Visitor>] {
					type Value = $struct_name;

					fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
						write!(formatter, "object {} ({})", stringify!($struct_name), stringify!($explanation))
					}

					fn visit_map<V>(self, mut map: V) -> Result<$struct_name, V::Error>
					where
						V: serde::de::MapAccess<'de>,
					{
						$(let mut $field_name = None;)*
						let mut errors = $crate::struct_deserialize::StructValidator::new();
						while let Some(key) = map.next_key()? {
							match key {
								$(Field::$field_name => {
									match map.next_value() {
										Ok(v) => {
											$field_name = v;
										},
										Err(e) => {
											errors.insert(stringify!($field_name), e.to_string());
										}
									}
								})*,
								_ => {
									let _ = match map.next_value::<serde::de::IgnoredAny>() {
										serde::export::Ok(__val) => __val,
										serde::export::Err(__err) => {
											return serde::export::Err(__err);
										}
									};
								}
							}
						}
						$(
							if $field_name.is_none() && ! errors.contains(stringify!($field_name)) {
								errors.insert(stringify!($field_name), "field is missing")
							}
						)*

						if (!errors.is_empty()) {
							return Err(serde::de::Error::custom(
								errors.to_json_string()
							));
						}
						$(
							let $field_name = $field_name.ok_or_else(
								|| serde::de::Error::missing_field(stringify!($field_name))
							)?;
						)*
						Ok($struct_name {
							$($field_name),*
						})
					}
				}
				const FIELDS: &'static [&'static str] = &[$(stringify!($field_name)),*];
				deserializer.deserialize_struct(
					stringify!($struct_name),
					FIELDS,
					[<$struct_name Visitor>]
				)
			}
		}
	}
}
}
