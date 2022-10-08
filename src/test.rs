#[cfg(test)]
mod tests {
	use crate::mock::{
        ExtBuilder, System, MaxLenCommitMessage,
    };
    
	#[test]
	fn check_pallet_settings() {
        ExtBuilder::default().build_and_execute(|| {            
			assert_eq!(System::block_number(), 0);

            let configs_max_len_msg = <MaxLenCommitMessage>::get();
            assert_eq!(configs_max_len_msg, 256, "max lex: {}", configs_max_len_msg);
        })
	}
}
