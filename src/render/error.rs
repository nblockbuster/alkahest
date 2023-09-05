// Contains the error mesh used when an object fails to render for whatever reason

use std::{io::Cursor, rc::Rc};

use glam::Mat4;
use tracing::warn;
use windows::Win32::Graphics::{
    Direct3D::{
        Fxc::{D3DCompileFromFile, D3DCOMPILE_DEBUG, D3DCOMPILE_SKIP_OPTIMIZATION},
        *,
    },
    Direct3D11::*,
    Dxgi::Common::*,
};

use crate::texture::{Texture, TextureHandle};

use super::{ConstantBuffer, DeviceContextSwapchain};

#[allow(unused)]
pub struct ErrorRenderer {
    vertex_buffer: ID3D11Buffer,
    vertex_count: usize,
    vertex_layout: ID3D11InputLayout,

    texture: Texture,
    vshader: ID3D11VertexShader,
    pshader: ID3D11PixelShader,

    scope: ConstantBuffer<AlkScopeError>,
}

impl ErrorRenderer {
    pub fn load(dcs: Rc<DeviceContextSwapchain>) -> Self {
        let matcap = unsafe {
            const MATCAP_DATA: &[u8] = include_bytes!("../../assets/textures/error.data");
            dcs.device
                .CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Width: 128 as _,
                        Height: 128 as _,
                        MipLevels: 1,
                        ArraySize: 1 as _,
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_SHADER_RESOURCE,
                        CPUAccessFlags: Default::default(),
                        MiscFlags: Default::default(),
                    },
                    Some(&D3D11_SUBRESOURCE_DATA {
                        pSysMem: MATCAP_DATA.as_ptr() as _,
                        SysMemPitch: 128 * 4,
                        ..Default::default()
                    } as _),
                )
                .expect("Failed to create error texture")
        };

        let matcap_view = unsafe {
            dcs.device
                .CreateShaderResourceView(
                    &matcap,
                    Some(&D3D11_SHADER_RESOURCE_VIEW_DESC {
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
                        Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
                            Texture2D: D3D11_TEX2D_SRV {
                                MostDetailedMip: 0,
                                MipLevels: 1,
                            },
                        },
                    }),
                )
                .expect("Failed to create error texture view")
        };

        let obj_data = include_bytes!("../../assets/models/error.obj");
        let reader = Cursor::new(obj_data);
        let obj = obj::ObjData::load_buf(reader).unwrap();

        let mut vertices = vec![];
        for vb in &obj.objects[0].groups[0].polys {
            for p in &vb.0 {
                let vi = p.0;
                let vni = p.2.unwrap_or_default();

                vertices.push([
                    obj.position[vi][0],
                    obj.position[vi][1],
                    obj.position[vi][2],
                    obj.normal[vni][0],
                    obj.normal[vni][1],
                    obj.normal[vni][2],
                ]);
            }
        }
        let vertex_buffer = unsafe {
            dcs.device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: (std::mem::size_of::<[f32; 6]>() * vertices.len()) as _,
                        Usage: D3D11_USAGE_IMMUTABLE,
                        BindFlags: D3D11_BIND_VERTEX_BUFFER,
                        ..Default::default()
                    },
                    Some(&D3D11_SUBRESOURCE_DATA {
                        pSysMem: vertices.as_ptr() as _,
                        ..Default::default()
                    }),
                )
                .expect("Failed to create error vertex buffer")
        };

        let mut vshader = None;
        let mut pshader = None;
        let mut errors = None;

        let flags = if cfg!(debug_assertions) {
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
        } else {
            0
        };
        unsafe {
            (
                D3DCompileFromFile(
                    w!("assets/shaders/error.hlsl"),
                    None,
                    None,
                    s!("VShader"),
                    s!("vs_5_0"),
                    flags,
                    0,
                    &mut vshader,
                    Some(&mut errors),
                )
                .expect("Failed to compile error vertex shader"),
                D3DCompileFromFile(
                    w!("assets/shaders/error.hlsl"),
                    None,
                    None,
                    s!("PShader"),
                    s!("ps_5_0"),
                    flags,
                    0,
                    &mut pshader,
                    Some(&mut errors),
                )
                .expect("Failed to compile error pixel shader"),
            )
        };

        if let Some(errors) = errors {
            let estr = unsafe {
                let eptr = errors.GetBufferPointer();
                std::slice::from_raw_parts(eptr.cast(), errors.GetBufferSize())
            };
            let errors = String::from_utf8_lossy(estr);
            warn!("{}", errors);
        }

        let vshader_blob = vshader.unwrap();
        let pshader_blob = pshader.unwrap();

        let (vshader, pshader) = unsafe {
            let vs_blob = std::slice::from_raw_parts(
                vshader_blob.GetBufferPointer() as *const u8,
                vshader_blob.GetBufferSize(),
            );
            let v2 = dcs
                .device
                .CreateVertexShader(vs_blob, None)
                .expect("Failed to load error vertex shader");
            let ps_blob = std::slice::from_raw_parts(
                pshader_blob.GetBufferPointer() as *const u8,
                pshader_blob.GetBufferSize(),
            );
            let v3 = dcs
                .device
                .CreatePixelShader(ps_blob, None)
                .expect("Failed to load error pixel shader");
            (v2, v3)
        };

        let vertex_layout = unsafe {
            let vs_blob = std::slice::from_raw_parts(
                vshader_blob.GetBufferPointer() as *const u8,
                vshader_blob.GetBufferSize(),
            );
            dcs.device
                .CreateInputLayout(
                    &[
                        D3D11_INPUT_ELEMENT_DESC {
                            SemanticName: s!("POSITION"),
                            SemanticIndex: 0,
                            Format: DXGI_FORMAT_R32G32B32_FLOAT,
                            InputSlot: 0,
                            AlignedByteOffset: 0,
                            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                            InstanceDataStepRate: 0,
                        },
                        D3D11_INPUT_ELEMENT_DESC {
                            SemanticName: s!("NORMAL"),
                            SemanticIndex: 0,
                            Format: DXGI_FORMAT_R32G32B32_FLOAT,
                            InputSlot: 0,
                            AlignedByteOffset: 12,
                            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                            InstanceDataStepRate: 0,
                        },
                    ],
                    vs_blob,
                )
                .expect("Failed to create error vertex layout")
        };

        Self {
            vertex_buffer,
            vertex_count: vertices.len(),
            vertex_layout,
            texture: Texture {
                view: matcap_view,
                handle: TextureHandle::Texture2D(matcap),
                format: crate::dxgi::DxgiFormat::R8G8B8A8_UNORM,
            },
            vshader,
            pshader,
            scope: ConstantBuffer::create(dcs, None).unwrap(),
        }
    }

    // TODO(cohae): Get this to work with the new renderer
    // pub fn draw(
    //     &self,
    //     renderer: &mut Renderer,
    //     transform: ID3D11Buffer,
    //     proj_view: Mat4,
    //     view: Mat4,
    // ) {
    //     self.scope
    //         .write(&AlkScopeError {
    //             proj_view,
    //             view,
    //             model: transform,
    //         })
    //         .unwrap();

    //     unsafe {
    //         dcs.context.IASetVertexBuffers(
    //             0,
    //             1,
    //             Some([Some(self.vertex_buffer.clone())].as_ptr()),
    //             Some([6 * 4].as_ptr()),
    //             Some(&0),
    //         );

    //         dcs.context
    //             .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

    //         dcs.context
    //             .VSSetConstantBuffers(7, Some(&[Some(self.scope.buffer().clone())]));

    //         dcs.context.IASetInputLayout(&self.vertex_layout);
    //         dcs.context.VSSetShader(&self.vshader, None);

    //         dcs.context.PSSetShader(&self.pshader, None);

    //         dcs.context
    //             .PSSetShaderResources(0, Some(&[Some(self.texture.view.clone())]));

    //         dcs.context.Draw(self.vertex_count as u32, 0);
    //     }
    // }
}

#[allow(unused)]
#[repr(C)]
struct AlkScopeError {
    pub proj_view: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}
