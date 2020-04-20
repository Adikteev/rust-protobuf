fn main() {
    protobuf_codegen_pure::Args::new().
        out_dir("/Users/geps/dev/lib/rust-protobuf/outdir").
        input("/Users/geps/dev/adikteev/bidder_adx/docs/protos/realtime-bidding.proto").
        includes(&["/Users/geps/dev/adikteev/bidder_adx/docs/protos/"]).
        customize(protobuf_codegen_pure::Customize {
            serde_derive: Some(true),
            ..Default::default()
        }).run().expect("protoc");
    // protoc_rust::Args::new().
    //     out_dir("/Users/geps/dev/lib/rust-protobuf/outdir2").
    //     input("/Users/geps/dev/adikteev/bidder_adx/docs/protos/messages.proto").
    //     includes(&["/Users/geps/dev/adikteev/bidder_adx/docs/protos/", "/Users/geps/dev/lib/rust-protobuf/proto/"]).
    //     customize(protobuf_codegen::Customize {
    //         serde_derive: Some(true),
    //         ..Default::default()
    //     }).run().expect("protoc");
}
