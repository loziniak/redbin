mod de;
mod error;
mod ser;

pub use crate::de::{from_bytes, Deserializer};
pub use crate::ser::{to_bytes, Serializer};

mod iconv_tools {
    use iconv::{Iconv, IconvError};
    use dyn_buf::VecBuf;

    const MIN_WRITE: usize = 4096;

    /// convert `input` from `from_encoding` to `to_encoding`
    pub fn iconv(c: &mut Iconv, input: &[u8]) -> std::result::Result<Vec<u8>, IconvError> {
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
    

}
