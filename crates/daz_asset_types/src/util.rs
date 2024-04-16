/// Silly macro for defining a C-style ZST enum that can be deserialized from
/// string values.
macro_rules! strenum {
	($EnumName:ident $($Enumerator:ident = $value:literal),+ $(,)?) => {
		#[allow(clippy::upper_case_acronyms)]
		#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
		pub enum $EnumName {
			#[default]
			$($Enumerator),+
		}

		::paste::paste! { struct [<$EnumName Visitor>]; }

		impl<'a> ::serde::de::Visitor<'a> for ::paste::paste!{ [<$EnumName Visitor>] } {
			type Value = $EnumName;

			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				f.write_str(stringify!($($value), +))
			}

			fn visit_str<E>(self, v: &str) -> Result<$EnumName, E>
			where E: serde::de::Error {
				match v {
					$(
						$value => Ok($EnumName::$Enumerator),
					)+
					other => Err(E::invalid_value(::serde::de::Unexpected::Str(other), &self)),
				}
			}
		}

		impl<'a> Deserialize<'a> for $EnumName {
			fn deserialize<D>(de: D) -> Result<Self, D::Error>
			where D: serde::Deserializer<'a> {
				de.deserialize_str(::paste::paste!{ [<$EnumName Visitor>] })
			}
		}
	};
}

pub(crate) use strenum;
