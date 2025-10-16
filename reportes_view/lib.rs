#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reportes_view {
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;
    use ink::prelude::collections::BTreeMap;
    use ink::env::call::FromAccountId;
    use marketplace_principal::{MarketplacePrincipalRef, Orden, Producto};




    #[ink(storage)]
    pub struct ReportesView {
        admin: AccountId,
        principal: AccountId,
    }

    impl ReportesView {
        /// Crea el contrato de reportes y fija la dirección del contrato principal.
        ///
        /// # Qué hace
        /// Guarda al caller como `admin` y setea `principal` con la cuenta del
        /// `MarketplacePrincipal` ya desplegado, que será consultado para obtener datos.
        ///
        /// # Parámetros
        /// - `principal`: `AccountId` del contrato principal.
        ///
        /// # Retornos
        /// - Nueva instancia de `ReportesView`.
        #[ink(constructor)]
        pub fn new(principal: AccountId) -> Self {
            let admin = Self::env().caller();
            Self { admin, principal }
        }

        /// Devuelve la cuenta `admin` y la dirección configurada del contrato principal.
        ///
        /// # Retornos
        /// - `(admin, principal)` como tupla de `AccountId`.
        #[ink(message)]
        pub fn info(&self) -> (AccountId, AccountId) {
            (self.admin, self.principal)
        }

        /// Actualiza la dirección del contrato principal. Solo puede hacerlo `admin`.
        ///
        /// # Parámetros
        /// - `nuevo`: `AccountId` del nuevo contrato principal.
        ///
        /// # Retornos
        /// - `Ok(())` si la operación fue exitosa.
        /// - `Err(())` si el caller no es el `admin`.
        #[ink(message)]
        pub fn set_principal(&mut self, nuevo: AccountId) -> Result<(), ()> {
            if self.env().caller() != self.admin {
                return Err(());
            }
            self.principal = nuevo;
            Ok(())
        }

        fn principal_ref(&self) -> MarketplacePrincipalRef {
            MarketplacePrincipalRef::from_account_id(self.principal)
        }


        /// Obtiene una copia de una orden del contrato principal, si existe.
        ///
        /// # Parámetros
        /// - `orden_id`: Identificador de la orden.
        ///
        /// # Retornos
        /// - `Some(Orden)` si la orden existe.
        /// - `None` si no existe.
        #[ink(message)]
        pub fn obtener_orden(&self, orden_id: u32) -> Option<Orden> {
            self.principal_ref().obtener_orden_pub(orden_id)
        }

        /// Devuelve las reputaciones acumuladas de un usuario.
        ///
        /// # Parámetros
        /// - `usuario`: `AccountId` del usuario a consultar.
        ///
        /// # Retornos
        /// - `Some((rep_como_comprador, rep_como_vendedor))` si el usuario existe.
        /// - `None` si el usuario no está registrado en el principal.
        #[ink(message)]
        pub fn obtener_reputaciones(&self, usuario: AccountId) -> Option<(u32, u32)> {
            self.principal_ref().obtener_reputaciones(usuario)
        }

        /// Lista todas las órdenes donde el usuario participa como comprador o vendedor.
        ///
        /// # Parámetros
        /// - `usuario`: `AccountId` del usuario a consultar.
        ///
        /// # Retornos
        /// - `Vec<Orden>` con copias de las órdenes encontradas (puede ser vacío).
        #[ink(message)]
        pub fn listar_ordenes_por_usuario(&self, usuario: AccountId) -> Vec<Orden> {
            self.principal_ref().listar_ordenes_por_usuario(usuario)
        }

        /// Top-5 vendedores por reputación como vendedores, a partir de una lista de candidatos.
        ///
        /// # Parámetros
        /// - `candidatos`: cuentas a evaluar.
        ///
        /// # Retornos
        /// - Vector de tamaño ≤ 5 con tuplas `(vendedor, reputacion_vendedor)` ordenado desc.
        #[ink(message)]
        pub fn top5_vendedores(&self, candidatos: Vec<AccountId>) -> Vec<(AccountId, u32)> {
            let principal = self.principal_ref();
            let mut datos: Vec<(AccountId, u32)> = candidatos
                .into_iter()
                .filter_map(|acc| principal.obtener_reputaciones(acc).map(|(_, rv)| (acc, rv))) //igual que los top5 compradores, pero ahora se queda con la reputación como vendedor, rv es la primera parte de la tupla
                .collect();
            datos.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))); // el cmp compara la segunda parte de la tupla (reputación) y en caso de empate compara la primera parte (AccountId)
            datos.truncate(5);
            datos
        }

        /// Top-5 compradores por reputación como compradores, a partir de una lista de candidatos.
        ///
        /// # Parámetros
        /// - `candidatos`: cuentas a evaluar.
        ///
        /// # Retornos
        /// - Vector de tamaño ≤ 5 con tuplas `(comprador, reputacion_comprador)` ordenado desc.
        #[ink(message)]
        pub fn top5_compradores(&self, candidatos: Vec<AccountId>) -> Vec<(AccountId, u32)> {
            let principal = self.principal_ref();
            let mut datos: Vec<(AccountId, u32)> = candidatos
                .into_iter()
                .filter_map(|acc| principal.obtener_reputaciones(acc).map(|(rc, _)| (acc, rc))) // se queda con la reputación como comprador, rc es la segunda parte de la tupla
                .collect();
            datos.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))); // Ordena por reputación desc, luego por id asc. a es una tupla (AccountId, u32)
            datos.truncate(5); // limita el vector a 5 elementos como máximo para el top 5 
            datos
        }

        /// Cantidad de órdenes por usuario en una lista dada.
        ///
        /// # Parámetros
        /// - `usuarios`: cuentas a consultar.
        ///
        /// # Retornos
        /// - Vector `(usuario, cantidad_de_ordenes)` para cada cuenta ingresada.
        #[ink(message)]
        pub fn cantidad_ordenes_por_usuario(&self, usuarios: Vec<AccountId>) -> Vec<(AccountId, u32)> {
            let principal = self.principal_ref();
            usuarios
                .into_iter() // iterador sobre las cuentas
                .map(|u| (u, principal.listar_ordenes_por_usuario(u).len() as u32)) // len() da usize, se convierte a u32
                .collect()
        }

        /// Calcula los productos más vendidos a partir de una lista de IDs de órdenes.
        ///
        /// # Parámetros
        /// - `ordenes_ids`: identificadores de órdenes a considerar.
        ///
        /// # Retornos
        /// - Vector `(producto_id, total_unidades)` ordenado por `total_unidades` desc.
        #[ink(message)]
        pub fn productos_mas_vendidos_desde(&self, ordenes_ids: Vec<u32>) -> Vec<(u32, u32)> {
            use ink::prelude::collections::BTreeMap;
            let principal = self.principal_ref();
            let mut mapa: BTreeMap<u32, u32> = BTreeMap::new();
            for oid in ordenes_ids {
                if let Some(o) = principal.obtener_orden_pub(oid) {
                    let entry = mapa.entry(o.producto_id).or_insert(0); // el entry es una referencia mutable de la cantidad actual de ese producto
                        *entry = entry.saturating_add(o.cantidad); // suma segura para evitar overflow

                }
            }
            let mut v: Vec<(u32, u32)> = mapa.into_iter().collect(); // convierte el mapa en un vector de tuplas
            v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))); // Ordena por cantidad desc, luego por id asc
            v
        }

        /// Devuelve estadísticas por categoría: total de ventas y promedio de reputación de vendedores.
        /// 
        /// - `categorias`: lista de categorías en bytes (Vec<u8>), se convierten a String.
        /// - Retorna: Vec de (categoría, total_ventas, promedio_reputación_vendedor).
        #[ink(message)]
        pub fn estadisticas_por_categoria(&self, categorias: Vec<Vec<u8>>) -> Vec<(String, u32, u32)> {
            let principal = self.principal_ref();
            let mut resultado: Vec<(String, u32, u32)> = Vec::new();

            for categoria_bytes in categorias { // cada categoría es un Vec<u8>, se convierte a String porque en el contrato principal las categorías son Strings
                let categoria_str = String::from_utf8(categoria_bytes).unwrap_or_default();

                // Productos en esa categoría
                let productos = principal.listar_productos_por_categoria(categoria_str.clone());

                // Total de ventas (sumamos cantidades de todas las órdenes de esos productos)
                let mut total_ventas: u32 = 0;

                // Conjunto de vendedores en esta categoría (usamos BTreeMap como "set")
                let mut vendedores_cat: BTreeMap<AccountId, ()> = BTreeMap::new();

                for p in productos {
                    vendedores_cat.entry(p.vendedor).or_insert(());
                    let ordenes = principal.listar_ordenes_por_producto(p.id);
                    for o in ordenes {
                        total_ventas = total_ventas.saturating_add(o.cantidad);
                    }
                }

                // Promedio de reputación de vendedores que publican en esta categoría
                let mut suma_rep: u32 = 0;
                let mut cant_rep: u32 = 0;
                for (vendedor, _) in vendedores_cat.into_iter() {
                    if let Some((_rep_comp, rep_vend)) = principal.obtener_reputaciones(vendedor) {
                        suma_rep = suma_rep.saturating_add(rep_vend);
                        cant_rep = cant_rep.saturating_add(1);
                    }
                }
                let promedio = suma_rep.checked_div(cant_rep).unwrap_or(0); //Tengo que hacer esto porque el clippy me lo pide, para evitar division por cero

                resultado.push((categoria_str, total_ventas, promedio));
            }

            resultado
        }

    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        #[ink::test]
        fn iniciar_set_principal() { // testea constructor, info y set_principal
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            let mut rv = ReportesView::new(acc.bob);
            let (admin, principal) = rv.info();
            assert_eq!(admin, acc.alice);
            assert_eq!(principal, acc.bob);
            assert!(rv.set_principal(acc.charlie).is_ok());
            let (_a2, p2) = rv.info();
            assert_eq!(p2, acc.charlie);
        }

        #[ink::test]
        fn admin_set() { // testea que solo el admin pueda cambiar el principal 
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            let mut rv = ReportesView::new(acc.bob);
            test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            assert!(rv.set_principal(acc.django).is_err());
        }
    }
}
