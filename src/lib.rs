mod de;
mod error;
mod ser;

pub use crate::de::{from_bytes, Deserializer};
pub use crate::error::{Error, Result};
pub use crate::ser::{to_bytes, Serializer};

mod iconv_tools {

	use iconv::{Iconv, IconvError};
	use dyn_buf::VecBuf;
	
	const MIN_WRITE: usize = 4096;
	
	pub struct Ic {
		ucs4_encoder: Iconv,
		ucs4_decoder: Iconv,
		ucs2_decoder: Iconv,
	}

	impl Ic {
	    pub fn new() -> Self {
	        Ic {
	            ucs4_encoder: encoder("UCS-4LE").unwrap(),
				ucs4_decoder: decoder("UCS-4LE").unwrap(),
				ucs2_decoder: decoder("UCS-2LE").unwrap(),
	        }
	    }
	    
		pub fn ucs4_encode(&mut self, input: &str) -> Result<Vec<u8>, IconvError> {
			encode(&mut self.ucs4_encoder, input)
		}
		
		pub fn ucs4_decode(&mut self, input: &[u8]) -> Result<String, IconvError> {
			decode(&mut self.ucs4_decoder, input)
		}
		
		pub fn ucs2_decode(&mut self, input: &[u8]) -> Result<String, IconvError> {
			decode(&mut self.ucs2_decoder, input)
		}
	}


	fn decoder(from_encoding: &str) -> Result<Iconv, IconvError> {
		Iconv::new(from_encoding, "UTF-8")
	}
	
	fn encoder(to_encoding: &str) -> Result<Iconv, IconvError> {
		Iconv::new("UTF-8", to_encoding)
	}
	
	/// convert `input` from `from_encoding` to `to_encoding`
	fn iconv(c: &mut Iconv, input: &[u8]) -> Result<Vec<u8>, IconvError> {
	    let mut read = 0;
	    let mut output = VecBuf::new(MIN_WRITE);
	    loop {
	        match c.convert(&input[read..], output.prepare_at_least(0)) {
	            Ok((r, w, _)) => {
	                output.commit(w);
	                if read >= input.len() {
	                    return Ok(output.into_vec());
	                }
	                read += r;
	            }
	            Err((r, w, IconvError::NotSufficientOutput)) => {
	                output.commit(w);
	                read += r;
	                output.grow(0);
	            }
	            Err((_, _, e)) => return Err(e),
	        }
	    }
	}

	/// convert `input` from UTF-8 to `encoding`
	fn encode(c: &mut Iconv, input: &str) -> Result<Vec<u8>, IconvError> {
	    iconv(c, input.as_bytes())
	}
	
	/// convert `input` from `encoding` to UTF-8
	fn decode(c: &mut Iconv, input: &[u8]) -> Result<String, IconvError> {
	    iconv(c, input).map(|v| unsafe { String::from_utf8_unchecked(v) })
	}

}
