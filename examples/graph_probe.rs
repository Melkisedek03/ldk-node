//! graph_probe — alat lab: baca state channel + network graph dari storage
//! node LDK yang SEDANG BERHENTI (jangan jalankan selagi node-nya hidup).
//!
//! Dipakai untuk verifikasi channel announced vs privat dari sudut pandang
//! node pemilik storage ini: apakah channel masuk network graph, dan apakah
//! node_announcement lawan (alias + alamat) tersimpan di graph lokal.
//!
//! Env (samakan dengan node yang diprobe):
//!   PROBE_STORAGE   - direktori storage node (wajib)
//!   PROBE_SEED_PATH - file seed node (bila node dibuat dengan seed file)
//!   PROBE_NETWORK   - "testnet" (default) | "mainnet"
//!   PROBE_ELECTRUM  - alamat electrum (default "127.0.0.1:60009")
//!   PROBE_LISTEN    - alamat listen dummy (default "127.0.0.1:19999";
//!                     probe TIDAK memanggil start(), tidak bind apa pun)
use ldk_node::bitcoin::Network;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::Builder;
use std::str::FromStr;

fn env(name: &str, default: &str) -> String {
	std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn main() {
	let storage = std::env::var("PROBE_STORAGE").expect("PROBE_STORAGE wajib diisi");
	let network_name = env("PROBE_NETWORK", "testnet");
	let network =
		if network_name == "mainnet" { Network::Nusacoin } else { Network::NusacoinTestnet };
	let electrum = env("PROBE_ELECTRUM", "127.0.0.1:60009");
	let listen = env("PROBE_LISTEN", "127.0.0.1:19999");

	let mut builder = Builder::new();
	builder.set_network(network);
	builder.set_chain_source_electrum(electrum, None);
	builder.set_storage_dir_path(storage);
	if let Ok(sp) = std::env::var("PROBE_SEED_PATH") {
		builder.set_entropy_seed_path(sp);
	}
	builder
		.set_listening_addresses(vec![SocketAddress::from_str(&listen).expect("listen addr")])
		.expect("set_listening_addresses");

	let node = builder.build().expect("[probe] build gagal");
	println!("[probe] node_id={}", node.node_id());

	println!("[probe] == channels (ChannelManager) ==");
	let chans = node.list_channels();
	if chans.is_empty() {
		println!("[probe] (tidak ada channel)");
	}
	for ch in &chans {
		println!(
			"[probe] id={:?} counterparty={} scid={:?} announced={} usable={} ready={} value={} nusan",
			ch.channel_id,
			ch.counterparty_node_id,
			ch.short_channel_id,
			ch.is_announced,
			ch.is_usable,
			ch.is_channel_ready,
			ch.channel_value_sats
		);
	}

	let graph = node.network_graph();
	println!("[probe] == network graph: channels ==");
	let scids = graph.list_channels();
	if scids.is_empty() {
		println!("[probe] (graph kosong - tidak ada channel terumumkan)");
	}
	for scid in &scids {
		match graph.channel(*scid) {
			Some(info) => println!(
				"[probe] scid={} node_one={:?} node_two={:?} capacity_nusan={:?} one_to_two={} two_to_one={}",
				scid,
				info.node_one,
				info.node_two,
				info.capacity_sats,
				info.one_to_two.is_some(),
				info.two_to_one.is_some()
			),
			None => println!("[probe] scid={} (info tidak ditemukan)", scid),
		}
	}

	println!("[probe] == network graph: nodes ==");
	let nodes = graph.list_nodes();
	if nodes.is_empty() {
		println!("[probe] (tidak ada node di graph)");
	}
	for nid in &nodes {
		match graph.node(nid) {
			Some(info) => match &info.announcement_info {
				Some(ai) => println!(
					"[probe] node={:?} channels={:?} alias={:?} addresses={:?} last_update={}",
					nid,
					info.channels,
					ai.alias().to_string(),
					ai.addresses(),
					ai.last_update()
				),
				None => println!(
					"[probe] node={:?} channels={:?} announcement_info=None (belum ada node_announcement)",
					nid, info.channels
				),
			},
			None => println!("[probe] node={:?} (info tidak ditemukan)", nid),
		}
	}
	println!("[probe] selesai");
}
