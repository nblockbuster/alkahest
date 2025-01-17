use anyhow::Context;
use crossbeam::channel::{self as mpsc, Receiver};
use destiny_pkg::TagHash;
use parking_lot::RwLock;
use std::time::Instant;
use std::{sync::Arc};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_INDEX_BUFFER, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC, D3D11_SUBRESOURCE_DATA,
    D3D11_USAGE_IMMUTABLE,
};

use crate::{
    dxgi::DxgiFormat,
    entity::{IndexBufferHeader, VertexBufferHeader},
    packages::package_manager,
    texture::Texture,
};

use super::{DeviceContextSwapchain, RenderData};

#[derive(PartialEq, Eq, Clone)]
pub enum LoadingThreadState {
    Idle,
    Loading {
        start_time: Instant,
        remaining: usize,
    },
}

pub static STATUS_TEXTURES: RwLock<LoadingThreadState> = RwLock::new(LoadingThreadState::Idle);
pub static STATUS_BUFFERS: RwLock<LoadingThreadState> = RwLock::new(LoadingThreadState::Idle);
// pub static STATUS_SHADERS: RwLock<LoadingThreadState> = RwLock::new(LoadingThreadState::Idle);

fn update_status(state: &RwLock<LoadingThreadState>, remaining: usize) {
    let status = state.read().clone();
    if remaining == 0 {
        *state.write() = LoadingThreadState::Idle;
    } else {
        match status {
            LoadingThreadState::Idle => {
                *state.write() = LoadingThreadState::Loading {
                    start_time: Instant::now(),
                    remaining,
                }
            }
            LoadingThreadState::Loading { start_time, .. } => {
                *state.write() = LoadingThreadState::Loading {
                    start_time,
                    remaining,
                }
            }
        }
    }
}

fn spawn_thread_textures(
    dcs: Arc<DeviceContextSwapchain>,
    data: Arc<RwLock<RenderData>>,
    rx: Receiver<TagHash>,
    name: &'static str,
) {
    std::thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            while let Ok(hash) = rx.recv() {
                if hash.is_valid() && !data.read().textures.contains_key(&hash) {
                    match Texture::load(&dcs, hash) {
                        Ok(t) => {
                            data.write().textures.insert(hash, t);
                        }
                        Err(e) => error!("Failed to load texture {hash}: {e}"),
                    }
                }

                update_status(&STATUS_TEXTURES, rx.len());
            }

            info!("Thread '{}' exited", name);
        })
        .unwrap();
}

pub fn thread_textures(
    dcs: Arc<DeviceContextSwapchain>,
    data: Arc<RwLock<RenderData>>,
) -> mpsc::Sender<TagHash> {
    let (tx, rx) = mpsc::unbounded::<TagHash>();

    spawn_thread_textures(dcs.clone(), data.clone(), rx.clone(), "Texture loader 1");
    spawn_thread_textures(dcs, data, rx, "Texture loader 2");

    tx
}

fn spawn_thread_buffers(
    dcs: Arc<DeviceContextSwapchain>,
    data: Arc<RwLock<RenderData>>,
    rx: Receiver<TagHash>,
    name: &'static str,
) {
    std::thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            while let Ok(hash) = rx.recv() {
                if hash.is_valid() {
                    info!("Loadin buffer {hash}");
                    if let Ok(entry) = package_manager().get_entry(hash) {
                        match (entry.file_type, entry.file_subtype) {
                            // Vertex buffer
                            (32, 4) => {
                                if data.read().vertex_buffers.contains_key(&hash) {
                                    continue;
                                }

                                match package_manager().read_tag(entry.reference) {
                                    Ok(vertex_data) => {
                                        let vertex_buffer_header = package_manager()
                                            .read_tag_struct::<VertexBufferHeader>(hash)
                                            .unwrap();

                                        let vertex_buffer = unsafe {
                                            dcs.device
                                                .CreateBuffer(
                                                    &D3D11_BUFFER_DESC {
                                                        ByteWidth: vertex_data.len() as _,
                                                        Usage: D3D11_USAGE_IMMUTABLE,
                                                        BindFlags: D3D11_BIND_VERTEX_BUFFER,
                                                        ..Default::default()
                                                    },
                                                    Some(&D3D11_SUBRESOURCE_DATA {
                                                        pSysMem: vertex_data.as_ptr() as _,
                                                        ..Default::default()
                                                    }),
                                                )
                                                .context("Failed to create vertex buffer")
                                                .unwrap()
                                        };

                                        data.write().vertex_buffers.insert(
                                            hash,
                                            (vertex_buffer, vertex_buffer_header.stride as u32),
                                        );
                                    }
                                    Err(e) => error!("Failed to load vertex buffer {hash}: {e}"),
                                }
                            }
                            // Index buffer
                            (32, 6) => {
                                if data.read().index_buffers.contains_key(&hash) {
                                    continue;
                                }

                                match package_manager().read_tag(entry.reference) {
                                    Ok(index_data) => {
                                        let index_buffer_header = package_manager()
                                            .read_tag_struct::<IndexBufferHeader>(hash)
                                            .unwrap();

                                        let index_buffer = unsafe {
                                            dcs.device
                                                .CreateBuffer(
                                                    &D3D11_BUFFER_DESC {
                                                        ByteWidth: index_data.len() as _,
                                                        Usage: D3D11_USAGE_IMMUTABLE,
                                                        BindFlags: D3D11_BIND_INDEX_BUFFER,
                                                        ..Default::default()
                                                    },
                                                    Some(&D3D11_SUBRESOURCE_DATA {
                                                        pSysMem: index_data.as_ptr() as _,
                                                        ..Default::default()
                                                    }),
                                                )
                                                .context("Failed to create index buffer")
                                                .unwrap()
                                        };

                                        data.write().index_buffers.insert(
                                            hash,
                                            (
                                                index_buffer,
                                                if index_buffer_header.is_32bit {
                                                    DxgiFormat::R32_UINT
                                                } else {
                                                    DxgiFormat::R16_UINT
                                                },
                                            ),
                                        );
                                    }
                                    Err(e) => error!("Failed to load vertex buffer {hash}: {e}"),
                                }
                            }
                            u => panic!("Unsupported mt loader buffer type {u:?}"),
                        }
                    }
                }

                update_status(&STATUS_BUFFERS, rx.len());
            }

            info!("Thread '{}' exited", name);
        })
        .unwrap();
}

pub fn thread_buffers(
    dcs: Arc<DeviceContextSwapchain>,
    data: Arc<RwLock<RenderData>>,
) -> mpsc::Sender<TagHash> {
    let (tx, rx) = mpsc::unbounded::<TagHash>();

    spawn_thread_buffers(dcs.clone(), data.clone(), rx.clone(), "Buffer loader 1");
    // spawn_thread_buffers(dcs, data, rx, "Buffer loader 2");

    tx
}
