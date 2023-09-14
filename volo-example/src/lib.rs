#![feature(impl_trait_in_assoc_type)]

use std::collections::HashMap;
use std::sync::Mutex;
use anyhow::anyhow;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use volo::FastStr;

pub struct S {
	kv: Mutex<HashMap<String, String>>,
	pub channels: Mutex<HashMap<String, Sender<String>>>
}

impl S {
	pub fn new() -> S {
		S {kv: Mutex::new(HashMap::new()), channels: Mutex::new(HashMap::new())}
	}
}
#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
	async fn get_item(&self, _req: volo_gen::volo::example::GetItemRequest) -> core::result::Result<volo_gen::volo::example::GetItemResponse, volo_thrift::AnyhowError> {
		let mut resp = volo_gen::volo::example::GetItemResponse{op: " ".into(), key: " ".into(), val: " ".into(), status: false};
		println!("收到！");
		let k = _req.key.to_string();
		let v = _req.val.to_string();
		match _req.op.as_str() {
			"set" => {
				resp.op = "set".to_string().into();
				//let k = _req.key.to_string();
				//let v = _req.val.to_string();
				let mut flag = 0;
				if self.kv.lock().unwrap().get(&k) == None {
					flag = 1;
				}
				match flag {
					1 => {
						self.kv.lock().unwrap().insert(k, v);
						//resp.val = v.clone().into();
						//resp.key = k.clone().into();
						resp.status = true;
					}
					0 => {
						resp.status = false;
					}
					_ => {
						resp.status = false;
					}
				}
			}
			"get" => {
				resp.op = "get".to_string().into();
				//let k = _req.key.to_string();
				match self.kv.lock().unwrap().get(&k)  {
					None => {
						resp.status = false;
					}
					Some(t) => {
						resp.val = t.clone().into();
						//resp.key = k.clone().into();
						resp.status = true;
					}
				}
			}
			"del" => {
				resp.op = "del".to_string().into();
				//let k = _req.key.to_string();
				match self.kv.lock().unwrap().remove(&k) {
					Some(t) => {
						resp.status = true;
					}
					None => {
						resp.status = false;
					}
				}
			}
			"ping" => {
				resp.op = "ping".to_string().into();
				resp.status = true;
			}
			"subscribe" => {
				//let k = _req.key.to_string();
				let (mut tx, mut rx) = broadcast::channel(16);
				resp.op = "subscribe".to_string().into();
				let mut is_exist = true;
				if let Some(tx) = self.channels.lock().unwrap().get(&k) {
					rx = tx.subscribe();
				}
				else {
					is_exist = false;
				}
				if !is_exist {
					self.channels.lock().unwrap().insert(k, tx);
				}
				let msg = rx.recv().await;
				match msg {
					Ok(m) => {
						resp.val = m.clone().into();
						resp.status = true;
					}
					Err(e) => {
						resp.status = false;
					}
				}
			}
			"publish" => {
				resp.op = "publish".to_string().into();
				//let k = _req.key.to_string();
				match self.channels.lock().unwrap().get(&k) {
					Some(tx) => {
						match tx.send(v) {
							Ok(n) => {
								resp.status = true;
								resp.val = FastStr::from((n as u8).to_string());
							}
							Err(e) => {
								resp.status = false;
							}
						}
					}
					None => {
						resp.status = false;
					}
				}
			}
			_ => {
				panic!("INVALID!");
			}
		}
		println!("处理完毕，送回");
		Ok(resp)
		//Ok(Default::default())
				}
}
pub struct FilterLayer;
impl<S> volo::Layer<S> for FilterLayer {
	type Service = FilterService<S>;

	fn layer(self, inner: S) -> Self::Service {
		FilterService(inner)
	}
}
#[derive(Clone)]
pub struct FilterService<S>(S);
#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for FilterService<S>
	where
		Req: std::fmt::Debug + Send + 'static,
		S: Send + 'static + volo::Service<Cx, Req> + Sync,
		Cx: Send + 'static,
		anyhow::Error: Into<S::Error>,
{
	async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
		let info = format!("{req:?}");
		let mut dirty = false;
		if info.contains("原神") || info.contains("傻逼") || info.contains("操你妈") {
			dirty = true;
		}
		match dirty {
			true => {
				Err(anyhow!("你怎么骂人呢？给我刷了牙再来").into())
			}
			false => {
				let resp =self.0.call(cx, req).await;
				resp
			}
		}
	}
}