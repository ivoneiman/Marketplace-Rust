/*

Los llamados que hago al contrato principal me conviene meterlos en el contrato de reportes_view en funciones privadas,
 y esas funciones privadas son las que tengo que usar en las funciones públicas
 para integrarlo con el contrato de reportes para replicar esa misma función (nada más que le pones config(test) o config(no-test),
  y lo que hay que hacer en la que  tenga config(test) es llamar a self.principal_fake.dameData (el método que sea, dameData es un ejemplo).
Y en la storage del contrato, como tengo la ref del contrat principal, agrego otro atributo que tenga arriba config test,
 y el atributo de la referencia del contrato principal se le pone config no test. 
Después cuando hago el test y creo el contrato de reportes, creo un struct fake con los métodos que se llamen dentro del mock y que retornen datas hardcodeadas en el test. 
*/

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reportes_view {
    use ink::env::call::FromAccountId;
    use ink::prelude::collections::BTreeMap;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    // Ref y tipos del contrato principal
    use marketplace_principal::{MarketplacePrincipalRef, Orden, Producto, EstadoOrden};

    
    // ========== Fake mínimo para tests ==========

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct PrincipalFake {
        // Si no se setean, se devuelven defaults logicos none ovacios
        rep_default: Option<(u32, u32)>,
        rep_por_usuario: BTreeMap<AccountId, (u32, u32)>,

        orden_por_id: BTreeMap<u32, Orden>,
        ordenes_por_usuario: BTreeMap<AccountId, Vec<Orden>>,
        ordenes_por_producto: BTreeMap<u32, Vec<Orden>>,

        productos_por_categoria: BTreeMap<String, Vec<Producto>>,
    }

    #[cfg(test)]
    impl PrincipalFake {
        // Setters para armar el escenario del test. Automatizados para terminar mas rapido de mockear
        pub fn with_default_rep(mut self, rep: Option<(u32, u32)>) -> Self {
            self.rep_default = rep;
            self
        }
        pub fn with_rep_for(mut self, user: AccountId, rep: (u32, u32)) -> Self {
            self.rep_por_usuario.insert(user, rep);
            self
        }
        pub fn with_orden(mut self, id: u32, orden: Orden) -> Self {
            self.orden_por_id.insert(id, orden);
            self
        }
        pub fn with_ordenes_de_usuario(mut self, user: AccountId, ordenes: Vec<Orden>) -> Self {
            self.ordenes_por_usuario.insert(user, ordenes);
            self
        }
        pub fn with_ordenes_de_producto(mut self, prod_id: u32, ordenes: Vec<Orden>) -> Self {
            self.ordenes_por_producto.insert(prod_id, ordenes);
            self
        }
        pub fn with_productos_de_categoria(mut self, categoria: String, productos: Vec<Producto>) -> Self {
            self.productos_por_categoria.insert(categoria, productos);
            self
        }

        // Métodos con la misma firma que el contrato principal usa en los helpers, como charlamos antes
        pub fn obtener_reputaciones(&self, who: AccountId) -> Option<(u32, u32)> {
            if let Some(rep) = self.rep_por_usuario.get(&who) {
                return Some(*rep);
            }
            // Si no hay por usuario, devolvemos default o (0,0)
            self.rep_default.or(Some((0, 0)))
        }

        pub fn obtener_orden_pub(&self, orden_id: u32) -> Option<Orden> {
            self.orden_por_id.get(&orden_id).cloned()
        }

        pub fn listar_ordenes_por_usuario(&self, usuario: AccountId) -> Vec<Orden> {
            self.ordenes_por_usuario.get(&usuario).cloned().unwrap_or_default()
        }

        pub fn listar_ordenes_por_producto(&self, producto_id: u32) -> Vec<Orden> {
            self.ordenes_por_producto.get(&producto_id).cloned().unwrap_or_default()
        }

        pub fn listar_productos_por_categoria(&self, categoria: String) -> Vec<Producto> {
            self.productos_por_categoria.get(&categoria).cloned().unwrap_or_default()
        }
    }



    // ========== Storage ==========

    #[ink(storage)]
    pub struct ReportesView {
        /// En build normal: guardamos la Address del contrato principal y generamos la referencia luego.
        #[cfg(not(test))]
        principal: AccountId,

        /// En tests: creamo un fake con datos hardcodeados.
        #[cfg(test)]
        principal_fake: PrincipalFake,
    }

    // ========== Constructores ==========

    impl ReportesView {
        /// Constructor real: recibe la cuenta del contrato principal.
        #[ink(constructor)]
        #[cfg(not(test))]
        pub fn new(principal: AccountId) -> Self {
            Self { principal }
        }

        /// Constructor para tests: crea fake
        #[cfg(test)]
        pub fn new_with_fake(principal_fake: PrincipalFake) -> Self {
            Self { principal_fake }
        }
    }

    // ========== Helpers privados (esta es la única forma de  acceder al contrto principal) ==========

    impl ReportesView {
        /// construye la referencia al principal desde la address almacenada.
        #[cfg(not(test))]
        fn principal_ref(&self) -> MarketplacePrincipalRef {
            MarketplacePrincipalRef::from_account_id(self.principal)
        }

        // --- 1) Reputaciones ---
        fn helper_obtener_reputaciones(&self, usuario: AccountId) -> Option<(u32, u32)> {
            #[cfg(not(test))]
            { self.principal_ref().obtener_reputaciones(usuario) }
            #[cfg(test)]
            { self.principal_fake.obtener_reputaciones(usuario) }
        }

        // --- 2) Ordenes ---
        fn helper_obtener_orden(&self, orden_id: u32) -> Option<Orden> {
            #[cfg(not(test))]
            { self.principal_ref().obtener_orden_pub(orden_id) }
            #[cfg(test)]
            { self.principal_fake.obtener_orden_pub(orden_id) }
        }

        fn helper_listar_ordenes_por_usuario(&self, usuario: AccountId) -> Vec<Orden> {
            #[cfg(not(test))]
            { self.principal_ref().listar_ordenes_por_usuario(usuario) }
            #[cfg(test)]
            { self.principal_fake.listar_ordenes_por_usuario(usuario) }
        }

        fn helper_listar_ordenes_por_producto(&self, producto_id: u32) -> Vec<Orden> {
            #[cfg(not(test))]
            { self.principal_ref().listar_ordenes_por_producto(producto_id) }
            #[cfg(test)]
            { self.principal_fake.listar_ordenes_por_producto(producto_id) }
        }

        // --- 3) Productos ---
        fn helper_listar_productos_por_categoria(&self, categoria: String) -> Vec<Producto> {
            #[cfg(not(test))]
            { self.principal_ref().listar_productos_por_categoria(categoria) }
            #[cfg(test)]
            { self.principal_fake.listar_productos_por_categoria(categoria) }
        }

        // --- Derivadas de reputaciones ---
        fn rep_como_comprador(&self, quien: AccountId) -> Option<u32> {
            self.helper_obtener_reputaciones(quien).map(|(comp, _vend)| comp)
        }
        fn rep_como_vendedor(&self, quien: AccountId) -> Option<u32> {
            self.helper_obtener_reputaciones(quien).map(|(_comp, vend)| vend)
        }

        // ========== Mensajes públicos (usan solo helpers) ==========

        /// Devuelve (rep_como_comprador, rep_como_vendedor) o None si no está registrado.
        #[ink(message)]
        pub fn reputacion_de(&self, who: AccountId) -> Option<(u32, u32)> {
            self.helper_obtener_reputaciones(who)
        }

        /// Solo reputación como comprador.
        #[ink(message)]
        pub fn reputacion_como_comprador(&self, who: AccountId) -> Option<u32> {
            self.rep_como_comprador(who)
        }

        /// Solo reputación como vendedor.
        #[ink(message)]
        pub fn reputacion_como_vendedor(&self, who: AccountId) -> Option<u32> {
            self.rep_como_vendedor(who)
        }

        /// Copia de una orden si existe.
        #[ink(message)]
        pub fn obtener_orden(&self, orden_id: u32) -> Option<Orden> {
            self.helper_obtener_orden(orden_id)
        }

        /// Todas las órdenes donde el usuario participa (comprador o vendedor).
        #[ink(message)]
        pub fn listar_ordenes_por_usuario(&self, usuario: AccountId) -> Vec<Orden> {
            self.helper_listar_ordenes_por_usuario(usuario)
        }

        /// Top-5 vendedores por reputación de vendedor, a partir de candidatos.
        #[ink(message)]
        pub fn top5_vendedores(&self, candidatos: Vec<AccountId>) -> Vec<(AccountId, u32)> {
            let mut datos: Vec<(AccountId, u32)> = candidatos
                .into_iter()
                .filter_map(|acc| self.rep_como_vendedor(acc).map(|rv| (acc, rv)))
                .collect();
            datos.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            datos.truncate(5);
            datos
        }

        /// Top-5 compradores por reputación de comprador, a partir de candidatos.
        #[ink(message)]
        pub fn top5_compradores(&self, candidatos: Vec<AccountId>) -> Vec<(AccountId, u32)> {
            let mut datos: Vec<(AccountId, u32)> = candidatos
                .into_iter()
                .filter_map(|acc| self.rep_como_comprador(acc).map(|rc| (acc, rc)))
                .collect();
            datos.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            datos.truncate(5);
            datos
        }

        /// Cantidad de órdenes por usuario (para cada usuario dado).
        #[ink(message)]
        pub fn cantidad_ordenes_por_usuario(&self, usuarios: Vec<AccountId>) -> Vec<(AccountId, u32)> {
            usuarios
                .into_iter()
                .map(|u| (u, self.helper_listar_ordenes_por_usuario(u).len() as u32))
                .collect()
        }

        /// Productos más vendidos a partir de una lista de IDs de órdenes.
        #[ink(message)]
        pub fn productos_mas_vendidos_desde(&self, ordenes_ids: Vec<u32>) -> Vec<(u32, u32)> {
            let mut mapa: BTreeMap<u32, u32> = BTreeMap::new();
            for oid in ordenes_ids {
                if let Some(o) = self.helper_obtener_orden(oid) {
                    let entry = mapa.entry(o.producto_id).or_insert(0);
                    *entry = entry.saturating_add(o.cantidad);
                }
            }
            let mut v: Vec<(u32, u32)> = mapa.into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            v
        }

        /// Estadísticas por categoría: (categoría, total_ventas, promedio_rep_vendedor).
        ///
        /// `categorias` viene como bytes porque ink no soporta String en los mensajes públicos.
        /// Cada categoría en el vector es una categoría distinta a consultar.
        #[ink(message)]
        pub fn estadisticas_por_categoria(&self, categorias: Vec<Vec<u8>>) -> Vec<(String, u32, u32)> {
            let mut resultado: Vec<(String, u32, u32)> = Vec::new();

            for categoria_bytes in categorias {
                let categoria_str = String::from_utf8(categoria_bytes).unwrap_or_default();

                // Productos en esa categoría
                let productos = self.helper_listar_productos_por_categoria(categoria_str.clone());

                // Total de ventas y conjunto de vendedores
                let mut total_ventas: u32 = 0;
                let mut vendedores_cat: BTreeMap<AccountId, ()> = BTreeMap::new();

                for p in productos {
                    vendedores_cat.entry(p.vendedor).or_insert(());
                    let ordenes = self.helper_listar_ordenes_por_producto(p.id);
                    for o in ordenes {
                        total_ventas = total_ventas.saturating_add(o.cantidad);
                    }
                }

                // Promedio reputación vendedores de la categoría
                let mut suma_rep: u32 = 0;
                let mut cant_rep: u32 = 0;
                for (vendedor, _) in vendedores_cat.into_iter() {
                    if let Some((_rep_comp, rep_vend)) = self.helper_obtener_reputaciones(vendedor) {
                        suma_rep = suma_rep.saturating_add(rep_vend);
                        cant_rep = cant_rep.saturating_add(1);
                    }
                }
                let promedio = suma_rep.checked_div(cant_rep).unwrap_or(0);

                resultado.push((categoria_str, total_ventas, promedio));
            }

            resultado
        }
    }



    // ========== Tests ==========

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

        fn orden_hardcodeada(id: u32, comprador: AccountId, vendedor: AccountId, producto_id: u32, cantidad: u32, estado: EstadoOrden) -> Orden {
            Orden {
                id,
                comprador,
                vendedor,
                producto_id,
                cantidad,
                estado,
                comprador_califico: false,
                vendedor_califico: false,
                cancelacion_solicitada_por: None,
                disputa_abierta: false,
            }
        }

        fn producto_hardcodeado(id: u32, nombre: &str, descripcion: &str, precio: Balance, cantidad: u32, categoria: &str, vendedor: AccountId) -> Producto {
            Producto {
                id,
                nombre: nombre.to_string(),
                descripcion: descripcion.to_string(),
                precio,
                cantidad,
                categoria: categoria.to_string(),
                vendedor,
            }
        }

        #[ink::test]
        fn reputaciones_funcionan_con_fake() {
            let user = AccountId::from([0xAA; 32]);

            let fake = PrincipalFake::default()
                .with_rep_for(user, (7, 12)); // (comprador, vendedor)

            let view = ReportesView::new_with_fake(fake);

            assert_eq!(view.reputacion_de(user), Some((7, 12)));
            assert_eq!(view.reputacion_como_comprador(user), Some(7));
            assert_eq!(view.reputacion_como_vendedor(user), Some(12));
        }

        #[ink::test]
        fn productos_mas_vendidos_desde_con_fake() {
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // (Si necesitás productos para otros tests, dejé el stub a mano)
            let _p10 = producto_hardcodeado(10, "Snack A", "Desc", 100u128, 999, "snacks", acc.bob);
            let _p11 = producto_hardcodeado(11, "Snack B", "Desc", 120u128, 999, "snacks", acc.bob);

            // Órdenes SIN Default, usando el stub (agregamos estado)
            let o1 = orden_hardcodeada(1, acc.alice, acc.bob,    10, 3, EstadoOrden::Recibida);
            let o2 = orden_hardcodeada(2, acc.alice, acc.bob,    10, 4, EstadoOrden::Recibida);
            let o3 = orden_hardcodeada(3, acc.bob,   acc.django, 11, 5, EstadoOrden::Recibida);

            let fake = PrincipalFake::default()
                .with_orden(1, o1.clone())
                .with_orden(2, o2.clone())
                .with_orden(3, o3.clone());

            let view = ReportesView::new_with_fake(fake);
            let ranking = view.productos_mas_vendidos_desde(vec![1, 2, 3]);

            assert_eq!(ranking, vec![(10, 7), (11, 5)]);
        }

        #[ink::test]
        fn top5_vendedores_con_fake() {
            let a = AccountId::from([0x01; 32]);
            let b = AccountId::from([0x02; 32]);
            let c = AccountId::from([0x03; 32]);

            let fake = PrincipalFake::default()
                .with_rep_for(a, (3, 10))  // vend 10
                .with_rep_for(b, (4, 15))  // vend 15
                .with_rep_for(c, (5, 12)); // vend 12

            let view = ReportesView::new_with_fake(fake);
            let top = view.top5_vendedores(vec![a, b, c]);

            assert_eq!(top, vec![(b, 15), (c, 12), (a, 10)]);
        }

        #[ink::test]
        fn helper_listar_ordenes_por_usuario_basico_y_vacio() {
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            let o1 = orden_hardcodeada(1, acc.alice, acc.bob,    10, 2, EstadoOrden::Recibida);
            let o2 = orden_hardcodeada(2, acc.alice, acc.charlie,11, 1, EstadoOrden::Recibida);
            let o3 = orden_hardcodeada(3, acc.bob,   acc.django, 10, 3, EstadoOrden::Recibida);

            let fake = PrincipalFake::default()
                .with_ordenes_de_usuario(acc.alice, vec![o1.clone(), o2.clone()])
                .with_ordenes_de_usuario(acc.bob,   vec![o3.clone()]);

            let view = ReportesView::new_with_fake(fake);

            // caso con órdenes
            let v_alice = view.helper_listar_ordenes_por_usuario(acc.alice);
            assert_eq!(v_alice.len(), 2);
            assert_eq!(v_alice[0].id, 1);
            assert_eq!(v_alice[1].id, 2);

            // caso vacío
            let v_eve = view.helper_listar_ordenes_por_usuario(acc.eve);
            assert!(v_eve.is_empty());
        }

        #[ink::test]
        fn helper_listar_ordenes_por_producto_basico_y_vacio() {
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            let o1 = orden_hardcodeada(1, acc.alice, acc.bob,    10, 2, EstadoOrden::Recibida);
            let o2 = orden_hardcodeada(2, acc.alice, acc.charlie,10, 4, EstadoOrden::Recibida);
            let o3 = orden_hardcodeada(3, acc.bob,   acc.django, 11, 3, EstadoOrden::Recibida);

            let fake = PrincipalFake::default()
                .with_ordenes_de_producto(10, vec![o1.clone(), o2.clone()])
                .with_ordenes_de_producto(11, vec![o3.clone()]);

            let view = ReportesView::new_with_fake(fake);

            let v_p10 = view.helper_listar_ordenes_por_producto(10);
            assert_eq!(v_p10.len(), 2);
            assert_eq!(v_p10.iter().map(|o| o.id).collect::<Vec<_>>(), vec![1, 2]);

            let v_p99 = view.helper_listar_ordenes_por_producto(99);
            assert!(v_p99.is_empty());
        }

        #[ink::test]
        fn helper_listar_productos_por_categoria_basico_y_vacio() {
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();
            let p1 = producto_hardcodeado(10, "Snack A", "Desc", 100u128, 50, "snacks", acc.bob);
            let p2 = producto_hardcodeado(11, "Snack B", "Desc", 120u128, 10, "snacks", acc.bob);
            let p3 = producto_hardcodeado(20, "Toy A",   "Desc", 200u128,  5, "toys",   acc.charlie);

            let fake = PrincipalFake::default()
                .with_productos_de_categoria("snacks".into(), vec![p1.clone(), p2.clone()])
                .with_productos_de_categoria("toys".into(),   vec![p3.clone()]);

            let view = ReportesView::new_with_fake(fake);

            let snacks = view.helper_listar_productos_por_categoria("snacks".into());
            assert_eq!(snacks.len(), 2);
            assert_eq!(snacks.iter().map(|p| p.id).collect::<Vec<_>>(), vec![10, 11]);

            let vacia = view.helper_listar_productos_por_categoria("books".into());
            assert!(vacia.is_empty());
        }

        #[ink::test]
        fn obtener_orden_some_y_none() {
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();
            let o1 = orden_hardcodeada(42, acc.alice, acc.bob, 10, 1, EstadoOrden::Recibida);

            let fake = PrincipalFake::default()
                .with_orden(42, o1.clone());

            let view = ReportesView::new_with_fake(fake);

            assert_eq!(view.obtener_orden(42).map(|o| o.id), Some(42));
            assert!(view.obtener_orden(7).is_none());
        }

        #[ink::test]
        fn listar_ordenes_por_usuario_public_devuelve_lo_esperado() {
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();
            let o1 = orden_hardcodeada(1, acc.alice, acc.bob, 10, 2, EstadoOrden::Recibida);
            let o2 = orden_hardcodeada(2, acc.alice, acc.bob, 11, 1, EstadoOrden::Recibida);

            let fake = PrincipalFake::default()
                .with_ordenes_de_usuario(acc.alice, vec![o1.clone(), o2.clone()]);

            let view = ReportesView::new_with_fake(fake);
            let lista = view.listar_ordenes_por_usuario(acc.alice);
            assert_eq!(lista.len(), 2);
            assert_eq!(lista[0].id, 1);
            assert_eq!(lista[1].id, 2);
        }

        #[ink::test]
        fn top5_compradores_con_empate_y_vacio() {
            // empate en reputación de comprador; orden secundario por AccountId asc
            let a = AccountId::from([0x01; 32]);
            let b = AccountId::from([0x02; 32]);
            let c = AccountId::from([0x03; 32]);

            let fake = PrincipalFake::default()
                .with_rep_for(a, (10, 1))  // comprador 10
                .with_rep_for(b, (10, 5))  // comprador 10 (empate)
                .with_rep_for(c, ( 7, 9)); // comprador 7

            let view = ReportesView::new_with_fake(fake);

            // candidatos vacíos
            assert!(view.top5_compradores(vec![]).is_empty());

            // con empate: b tiene AccountId mayor que a → a debería ir antes que b con mismo score
            let top = view.top5_compradores(vec![b, c, a]);
            assert_eq!(top.len(), 3);
            assert_eq!(top[0], (a, 10));
            assert_eq!(top[1], (b, 10));
            assert_eq!(top[2], (c, 7));
        }

        #[ink::test]
        fn cantidad_ordenes_por_usuario_multiples_y_ceros() {
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();
            let o1 = orden_hardcodeada(1, acc.alice, acc.bob, 10, 1, EstadoOrden::Recibida);
            let o2 = orden_hardcodeada(2, acc.alice, acc.bob, 11, 2, EstadoOrden::Recibida);

            let fake = PrincipalFake::default()
                .with_ordenes_de_usuario(acc.alice, vec![o1.clone(), o2.clone()]);
            let view = ReportesView::new_with_fake(fake);

            let res = view.cantidad_ordenes_por_usuario(vec![acc.alice, acc.bob, acc.eve]);
            // pasamos tres usuarios: alice(2), bob(0), eve(0)
            assert_eq!(res.len(), 3);
            assert!(res.contains(&(acc.alice, 2)));
            assert!(res.contains(&(acc.bob,   0)));
            assert!(res.contains(&(acc.eve,   0)));
        }

        #[ink::test]
        fn estadisticas_por_categoria_casos_borde_y_normal() {
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // categoría "snacks" con 2 productos, órdenes y reputaciones de vendedores
            let p10 = producto_hardcodeado(10, "Snack A", "Desc", 100u128, 100, "snacks", acc.bob);
            let p11 = producto_hardcodeado(11, "Snack B", "Desc", 120u128, 100, "snacks", acc.charlie);

            let o1 = orden_hardcodeada(1, acc.alice, acc.bob,     10, 3, EstadoOrden::Recibida);
            let o2 = orden_hardcodeada(2, acc.alice, acc.charlie, 11, 5, EstadoOrden::Recibida);

            // categoría "empty" sin productos
            let fake = PrincipalFake::default()
                .with_productos_de_categoria("snacks".into(), vec![p10.clone(), p11.clone()])
                .with_productos_de_categoria("empty".into(),  vec![])
                .with_ordenes_de_producto(10, vec![o1.clone()])
                .with_ordenes_de_producto(11, vec![o2.clone()])
                // reputaciones de vendedores para promedio
                .with_rep_for(acc.bob,     (4, 10))  // rep vendedor 10
                .with_rep_for(acc.charlie, (3, 20)); // rep vendedor 20

            let view = ReportesView::new_with_fake(fake);

            // consultamos dos categorías: "snacks" y "empty"
            let out = view.estadisticas_por_categoria(vec![
                b"snacks".to_vec(),
                b"empty".to_vec(),
            ]);

            // Deben venir dos resultados, en el mismo orden
            assert_eq!(out.len(), 2);

            // "snacks": total_ventas = 3 + 5 = 8 ; promedio_rep_vendedor = (10 + 20)/2 = 15
            assert_eq!(out[0].0, String::from("snacks"));
            assert_eq!(out[0].1, 8);
            assert_eq!(out[0].2, 15);

            // "empty": sin productos → total_ventas = 0; cant_rep = 0 -> promedio = 0
            assert_eq!(out[1].0, String::from("empty"));
            assert_eq!(out[1].1, 0);
            assert_eq!(out[1].2, 0);
        }

    }
}
