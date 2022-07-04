use futures_util::pin_mut;
use papyrus_lib::config::load_config;
use papyrus_lib::starknet::BlockNumber;
use papyrus_lib::sync::CentralSource;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let config = load_config("config/config.ron").unwrap();
    let mut state_marker = BlockNumber(200);
    let last_block_number = BlockNumber(203);

    let central_source = CentralSource::new(config.central).unwrap();
    let header_stream = central_source
        .stream_state_updates(state_marker, last_block_number)
        .fuse();
    pin_mut!(header_stream);
    while let Some(Ok((block_number, _state_difff))) = header_stream.next().await {
        assert!(
            state_marker == block_number,
            "Expected block number ({}) does not match the result ({}).",
            state_marker.0,
            block_number.0
        );
        state_marker = state_marker.next();
    }
}
