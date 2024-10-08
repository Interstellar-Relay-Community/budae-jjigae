/*
 * SPDX-FileCopyrightText: 2024 Aumetra Weisman <aumetra@cryptolab.net>
 *
 * SPDX-License-Identifier: AGPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//use crate::{kv_storage, mrf_wit::v1::fep::mrf::keyvalue};

use slab::Slab;
use triomphe::Arc;
use wasmtime::{
    component::{Resource, ResourceTable},
    Engine, Store,
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

//pub struct KvContext {
//    pub module_name: Option<String>,
//    pub storage: Arc<kv_storage::BackendDispatch>,
//    pub buckets: Slab<kv_storage::BucketBackendDispatch>,
//}

//impl KvContext {
//    #[inline]
//    pub fn get_bucket(
//        &self,
//        rep: &Resource<keyvalue::Bucket>,
//    ) -> &kv_storage::BucketBackendDispatch {
//        &self.buckets[rep.rep() as usize]
//    }
//}

pub struct Context {
//    pub kv_ctx: KvContext,
    pub resource_table: ResourceTable,
    pub wasi_ctx: WasiCtx,
}

impl WasiView for Context {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

#[inline]
pub fn construct_store(
    engine: &Engine,
//    storage: Arc<kv_storage::BackendDispatch>,
) -> Store<Context> {
    let wasi_ctx = WasiCtxBuilder::new()
        .allow_ip_name_lookup(false)
        .allow_tcp(false)
        .allow_udp(false)
        .build();

    Store::new(
        engine,
        Context {
//            kv_ctx: KvContext {
//                module_name: None,
//                storage,
//                buckets: Slab::new(),
//            },
            resource_table: ResourceTable::new(),
            wasi_ctx,
        },
    )
}
