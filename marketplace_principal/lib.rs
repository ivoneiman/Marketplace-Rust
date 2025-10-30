
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace_principal {
    // Importa los derive macros y tipos
    use ink::storage::traits::StorageLayout;
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    /// Estructura principal del contrato Marketplace.
    #[ink(storage)]
    pub struct MarketplacePrincipal {
        /// Mapeo de usuarios registrados (por dirección).
        usuarios: Mapping<AccountId, Usuario>,
        /// Lista de productos publicados.
        productos: Vec<Producto>,
        /// Lista de órdenes generadas.
        ordenes: Vec<Orden>,
    }

    impl MarketplacePrincipal {
        /// Crea una nueva instancia vacía del marketplace.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                usuarios: Mapping::default(),
                productos: Vec::new(),
                ordenes: Vec::new(),
            }
        }

        /// Registra al caller como usuario del sistema con el rol indicado.
        ///
        /// # Qué hace
        /// Crea una entrada en el mapeo `usuarios` asociada a la cuenta del caller,
        /// inicializando las reputaciones en 0. Si la cuenta ya estaba registrada, falla.
        ///
        /// # Parámetros
        /// - `rol`: Rol a asignar al usuario (`Comprador`, `Vendedor` o `Ambos`).
        ///
        /// # Retornos
        /// - `Ok(())` si el registro se realizó correctamente.
        /// - `Err(SistemaError::UsuarioExistente)` si la cuenta del caller ya estaba registrada.
        ///
        /// # Notas
        /// - Esta función solo crea el usuario. No otorga permisos especiales
        ///   por fuera de los definidos por el rol asignado.

        #[ink(message)]
        pub fn registrar_usuario(&mut self, rol: RolUsuario) -> Result<(), SistemaError> {
            self.registrar_usuario_interno(rol)
        }

        /// Consulta si una cuenta dada está registrada como usuario del marketplace.
        ///
        /// # Qué hace
        /// Verifica la existencia de la cuenta en el mapeo `usuarios`.
        ///
        /// # Parámetros
        /// - `usuario`: `AccountId` de la cuenta a consultar.
        ///
        /// # Retornos
        /// - `true` si `usuario` figura en el mapeo `usuarios`.
        /// - `false` en caso contrario.
        ///
        /// # Notas
        /// - Es una lectura pública que no requiere que el caller esté registrado.
        #[ink(message)]
        pub fn esta_registrado(&self, usuario: AccountId) -> bool {
            self.usuarios.contains(&usuario)
        }

        /// Devuelve la información pública de un usuario, si está registrado.
        ///
        /// # Qué hace
        /// Lee del mapeo `usuarios` y retorna una copia del struct `Usuario`
        /// correspondiente a la cuenta solicitada.
        ///
        /// # Parámetros
        /// - `usuario`: `AccountId` de la cuenta a consultar.
        ///
        /// # Retornos
        /// - `Some(Usuario)` si la cuenta está registrada.
        /// - `None` si la cuenta no existe en el sistema.
        ///
        /// # Notas
        /// - Es una lectura pública; no valida el rol del caller.
        #[ink(message)]
        pub fn obtener_usuario(&self, usuario: AccountId) -> Option<Usuario> {
            self.usuarios.get(&usuario)
        }

        /// Lógica interna para registrar un usuario.
        /// 
        /// - **Valida:** que el caller no esté ya registrado; de lo contrario `UsuarioExistente`.
        /// - **Efectos:** inserta en `usuarios` con reputaciones inicializadas en 0.
        /// - **Errores:** `UsuarioExistente`.
        fn registrar_usuario_interno(&mut self, rol: RolUsuario) -> Result<(), SistemaError> {
            let usuario_llamador = self.env().caller();
            // Verifica si el usuario es existente
            if self.usuarios.contains(&usuario_llamador) { 
                return Err(SistemaError::UsuarioExistente);
            }
            // Si no existe, crea un nuevo usuario
            let nuevo_usuario = Usuario {
                direccion: usuario_llamador,
                rol,
                reputacion_como_comprador: 0,
                reputacion_como_vendedor: 0,
            };
            self.usuarios.insert(usuario_llamador, &nuevo_usuario);
            Ok(())
        }

        /// Permite al caller cambiar su propio rol (p. ej., de `Comprador` a `Vendedor`).
        ///
        /// # Qué hace
        /// Valida que el caller esté registrado y que la transición de rol sea válida.
        /// Luego actualiza el rol y emite un evento `RolActualizado`.
        ///
        /// # Parámetros
        /// - `nuevo_rol`: Rol deseado (`Comprador`, `Vendedor` o `Ambos`).
        ///
        /// # Retornos
        /// - `Ok(())` si el cambio se aplicó correctamente.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si la transición pedida no es válida
        ///   (por ejemplo, pedir exactamente el mismo rol actual).
        ///
        /// # Efectos colaterales
        /// - Emite `RolActualizado { cuenta, rol_anterior, rol_nuevo }`.
        ///
        /// # Notas
        /// - Las reglas de transición se validan con `verificar_puede_cambiar_rol`.
        #[ink(message)]
        pub fn modificar_rol_usuario(&mut self,nuevo_rol: RolUsuario,) -> Result<(), SistemaError> {
            self.modificar_rol_usuario_interno(nuevo_rol)
        }
        fn modificar_rol_usuario_interno(&mut self,nuevo_rol: RolUsuario,) -> Result<(), SistemaError> {
            let usuario_llamador = self.env().caller();
            // Verifica que el usuario esté registrado
            self.verificar_registro(usuario_llamador)?;
            
            // Verifica que el usuario quiera cambiar a un rol que no es el rol actual
            self.verificar_puede_cambiar_rol(usuario_llamador, nuevo_rol.clone())?;

            // Actualiza el rol del usuario
            let mut usuario = self.usuarios.get(&usuario_llamador)
                .ok_or(SistemaError::UsuarioNoRegistrado)?;

            let rol_anterior = usuario.rol.clone(); // Guarda el rol viejo para el evento
            
            usuario.rol = nuevo_rol.clone();
            self.usuarios.insert(usuario_llamador, &usuario);

            //Evento
            self.env().emit_event(RolActualizado {
                cuenta: usuario_llamador,
                rol_anterior,
                rol_nuevo: nuevo_rol,
            });

            Ok(())
        }


        /// Publica un nuevo producto a nombre del caller (debe tener rol de vendedor).
        ///
        /// # Qué hace
        /// Valida que el caller esté registrado y tenga permiso de vendedor,
        /// y que la `cantidad` sea mayor a cero. Luego agrega un `Producto` a la lista,
        /// asignándole un `id` incremental y emite `ProductoPublicado`.
        ///
        /// # Parámetros
        /// - `nombre`: Nombre del producto.
        /// - `descripcion`: Descripción corta.
        /// - `precio`: Precio unitario (`Balance` del entorno Ink!).
        /// - `cantidad`: Stock inicial disponible (debe ser > 0).
        /// - `categoria`: Categoría libre asociada al producto.
        ///
        /// # Retornos
        /// - `Ok(())` si el producto fue publicado.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no tiene rol de vendedor.
        /// - `Err(SistemaError::CantidadInsuficiente)` si `cantidad == 0`.
        ///
        /// # Efectos colaterales
        /// - Emite `ProductoPublicado { vendedor, producto_id }`.
        ///
        /// # Notas
        /// - El `producto_id` es `self.productos.len()` previo al push.
        /// - La verificación de permisos usa `verificar_rol(caller, Vendedor)`.
        #[ink(message)]
        pub fn publicar_producto(
            &mut self,
            nombre: String,
            descripcion: String,
            precio: Balance,
            cantidad: u32,
            categoria: String,
        ) -> Result<(), SistemaError> {
            self.crear_producto_seguro(nombre, descripcion, precio, cantidad, categoria)
        }
        /// Lógica interna para validar y agregar un producto.
        /// 
        /// - **Valida:** caller registrado (`UsuarioNoRegistrado`), rol Vendedor/Ambos (`NoEsRolCorrecto`), `cantidad > 0` (`CantidadInsuficiente`).
        /// - **Efectos:** llama a `agregar_producto(...)`.
        /// - **Errores:** `UsuarioNoRegistrado`, `NoEsRolCorrecto`, `CantidadInsuficiente`.
        fn crear_producto_seguro(
            &mut self,
            nombre: String,
            descripcion: String,
            precio: Balance,
            cantidad: u32,
            categoria: String,
        ) -> Result<(), SistemaError> {
            let vendedor = self.env().caller();
            // Verifica que el vendedor esté registrado y tenga el rol adecuado
            self.verificar_registro(vendedor)?;
            self.verificar_rol(vendedor, RolUsuario::Vendedor)?;
            // Verifica que la cantidad sea válida
            self.verificar_cantidad(cantidad)?;
            // Agrega el producto al marketplace
            self.agregar_producto(nombre, descripcion, precio, cantidad, categoria, vendedor)
        }

        
        /// Lista todos los productos publicados por el caller (debe ser Vendedor o Ambos).
        ///
        /// # Qué hace
        /// Verifica que el caller esté registrado y tenga permiso de vendedor.
        /// Luego filtra `self.productos` devolviendo solo los que pertenecen al caller.
        ///
        /// # Parámetros
        /// _(ninguno)_
        ///
        /// # Retornos
        /// - `Ok(Vec<Producto>)` con los productos del caller.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no tiene rol de vendedor/ambos.
        /// - `Err(SistemaError::ProductosVacios)` si el caller no tiene publicaciones.
        ///
        /// # Notas
        /// - Internamente delega en `listar_productos_interno(caller)`.
        #[ink(message)]
        pub fn listar_mis_productos(&self) -> Result<Vec<Producto>, SistemaError> {
            let yo = self.env().caller();
            self.listar_productos_interno(yo)
        }
        /// Interna: valida que `vendedor` exista y tenga rol de Vendedor/Ambos,
        /// 
        /// - **Valida:** `verificar_rol(vendedor, Vendedor)` (propaga `UsuarioNoRegistrado` o `NoEsRolCorrecto`).
        /// - **Retorna:** productos del vendedor o `ProductosVacios` si no tiene publicaciones.
        fn listar_productos_interno(&self, vendedor: AccountId) -> Result<Vec<Producto>, SistemaError> {
            // Valida registro + rol; verificar rol ya devuelve UsuarioNoRegistrado o NoEsRolCorrecto
            self.verificar_rol(vendedor, RolUsuario::Vendedor)?;

            // Filtra los productos pertenecientes al vendedor
            let productos_vendedor: Vec<Producto> = self
                .productos
                .iter()
                .filter(|p| p.vendedor == vendedor)
                .cloned()
                .collect();

            if productos_vendedor.is_empty() {
                return Err(SistemaError::ProductosVacios);
            }
            Ok(productos_vendedor)
        }

        /// Lista todos los productos publicados por un vendedor específico (lectura pública).
        ///
        /// # Qué hace
        /// Recorre `self.productos` y devuelve aquellos cuyo `vendedor == vendedor` recibido.
        /// No valida registro ni rol del `vendedor` (consulta pública).
        ///
        /// # Parámetros
        /// - `vendedor`: `AccountId` del vendedor a consultar.
        ///
        /// # Retornos
        /// - `Ok(Vec<Producto>)` con los productos del vendedor.
        /// - `Err(SistemaError::ProductosVacios)` si el vendedor no tiene publicaciones.
        ///
        /// # Notas
        /// - Usa `listar_productos_por_vendedor_interno` para realizar la búsqueda.

        #[ink(message)]
        pub fn listar_productos_por_vendedor(&self, vendedor: AccountId) -> Result<Vec<Producto>, SistemaError> {
            self.listar_productos_por_vendedor_interno(vendedor)
        }
        /// Interna usada por el mensaje público de lectura abierta.
        /// 
        /// - **No** valida rol ni registro del `vendedor` (consulta pública).
        /// - **Retorna:** todos los productos del `vendedor` o `ProductosVacios` si no hay.
        pub fn listar_productos_por_vendedor_interno(&self, vendedor: AccountId) -> Result<Vec<Producto>, SistemaError> {
            let productos: Vec<Producto> = self.productos.iter().filter(|p| p.vendedor == vendedor).cloned().collect();
            if productos.is_empty() {
                return Err(SistemaError::ProductosVacios);
            }
            Ok(productos)
        }



        /// Crea una orden de compra a nombre del caller (debe ser Comprador o Ambos).
        ///
        /// # Qué hace
        /// Valida que el caller esté registrado y pueda comprar, que la `cantidad` sea > 0,
        /// que el `producto_id` exista y que haya stock suficiente. Descuenta el stock del
        /// producto y agrega una `Orden` nueva en estado `Pendiente`.
        ///
        /// # Parámetros
        /// - `producto_id`: Identificador del producto a comprar.
        /// - `cantidad`: Unidades solicitadas (debe ser > 0).
        ///
        /// # Retornos
        /// - `Ok(u32)` con el `orden_id` creado.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no puede comprar.
        /// - `Err(SistemaError::CantidadInsuficiente)` si `cantidad == 0`.
        /// - `Err(SistemaError::ProductosVacios)` si el producto no existe.
        /// - `Err(SistemaError::StockInsuficiente)` si no hay stock suficiente.
        ///
        /// # Efectos colaterales
        /// - Descuenta `cantidad` del stock del producto.
        /// - Inserta una `Orden` en `self.ordenes` con estado `Pendiente`.
        ///
        /// # Notas
        /// - La verificación de stock se hace antes del préstamo mutable.
        /// - El `orden_id` es `self.ordenes.len()` previo al push.

        #[ink(message)]
        pub fn crear_orden(&mut self, producto_id: u32, cantidad: u32) -> Result<u32, SistemaError> {
            self.crear_nueva_orden(producto_id, cantidad)
        }
        /// Crea una orden de compra descontando stock de forma segura.
        /// 
        /// - **Valida:** caller registrado y con permiso de compra (`UsuarioNoRegistrado`, `NoEsRolCorrecto`), `cantidad > 0` (`CantidadInsuficiente`), existencia del producto (`ProductosVacios`), stock suficiente (`StockInsuficiente`).
        /// - **Efectos:** descuenta `cantidad` del producto y pushea una nueva `Orden` en estado `Pendiente`.
        /// - **Notas:** usa un borrow inmutable breve para leer/validar y luego uno mutable para descontar stock.
        /// - **Errores:** ver arriba.
        fn crear_nueva_orden(&mut self, producto_id: u32, cantidad: u32) -> Result<u32, SistemaError> {
            let comprador = self.env().caller();
            
            // Validación temprana: verificar registro antes de cualquier otra operación
            self.verificar_registro(comprador)?;
            self.verificar_puede_comprar(comprador)?;
            
            // Validar que la cantidad solicitada sea válida
            self.verificar_cantidad(cantidad)?;
            
            // Buscar el producto en modo inmutable (solo lectura)
            //    Usamos un bloque para que el borrow inmutable dure poco
            let vendedor = {
                // Buscar el producto por id en el vector de productos
                let producto_ref = self.productos.iter().find(|p| p.id == producto_id)
                    .ok_or(SistemaError::ProductosVacios)?;   // Error si no existe ese id

                // Validar que el stock alcanzaba para la cantidad pedida
                self.verificar_stock_disponible(producto_ref, cantidad)?;

                // Guardar el vendedor en una variable independiente
                // (copiamos el AccountId, no un borrow)
                producto_ref.vendedor
            };
            
            // 👉 En este punto, el borrow inmutable de producto_ref ya terminó
            //    porque el bloque {...} cerró. Esto libera el préstamo inmutable
            //    y nos permite pedir ahora un préstamo mutable.

            // Obtener el producto en modo mutable para descontar stock
            let producto = self.obtener_producto_mut(producto_id)?;
            producto.cantidad = producto.cantidad.saturating_sub(cantidad);
            // `saturating_sub` asegura que nunca va a dar underflow (siempre >= 0).
            //Igual ya validamos stock antes, pero esto es más seguro.
            
            // 4) Crear la orden con todos los datos (comprador, vendedor, producto, cantidad)
            self.crear_y_emitir_orden(comprador, vendedor, producto_id, cantidad)
        }

        /// Permite al vendedor marcar una orden como `Enviada`.
        ///
        /// # Qué hace
        /// Verifica que el caller esté registrado, que sea el **vendedor** de la orden
        /// y que la transición de estado sea válida (`Pendiente → Enviada`).
        /// Luego actualiza el estado de la orden.
        ///
        /// # Parámetros
        /// - `orden_id`: Identificador de la orden a actualizar.
        ///
        /// # Retornos
        /// - `Ok(())` si el cambio se realizó.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::OrdenNoExiste)` si el `orden_id` no es válido.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no es el vendedor de la orden.
        /// - `Err(SistemaError::EstadoInvalido)` si la transición no es válida.
        ///
        /// # Notas
        /// - Internamente delega en `actualizar_estado_orden(orden_id, EstadoOrden::Enviada)`.

        #[ink(message)]
        pub fn marcar_orden_como_enviada(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            self.actualizar_estado_orden(orden_id, EstadoOrden::Enviada)
        }

        /// Permite al comprador marcar una orden como `Recibida`.
        ///
        /// # Qué hace
        /// Verifica que el caller esté registrado, que sea el **comprador** de la orden
        /// y que la transición sea válida (`Enviada → Recibida`). Luego actualiza el estado.
        ///
        /// # Parámetros
        /// - `orden_id`: Identificador de la orden a actualizar.
        ///
        /// # Retornos
        /// - `Ok(())` si el cambio se realizó.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::OrdenNoExiste)` si el `orden_id` no es válido.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no es el comprador de la orden.
        /// - `Err(SistemaError::EstadoInvalido)` si la transición no es válida.
        ///
        /// # Notas
        /// - Internamente delega en `actualizar_estado_orden(orden_id, EstadoOrden::Recibida)`.

        #[ink(message)]
        pub fn marcar_como_recibida(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            self.actualizar_estado_orden(orden_id, EstadoOrden::Recibida)
        }

        /// Actualiza el estado de una orden aplicando permisos y reglas de transición.
        /// 
        /// - **Valida:** caller registrado, pertenencia/permiso según `nuevo_estado` (vendedor para `Enviada`, comprador para `Recibida`), existencia de la orden, y transición válida (`Pendiente→Enviada`, `Enviada→Recibida`).
        /// - **Efectos:** cambia `orden.estado` al `nuevo_estado`.
        /// - **Errores:** `UsuarioNoRegistrado`, `OrdenNoExiste`, `NoEsRolCorrecto`, `EstadoInvalido`.
        fn actualizar_estado_orden(&mut self, orden_id: u32, nuevo_estado: EstadoOrden) -> Result<(), SistemaError> {
            let caller = self.env().caller();
            self.verificar_registro(caller)?;
            // Primero obten la orden de forma inmutable para verificar el permiso
            {
                let orden_ref = self.ordenes.get(orden_id as usize).ok_or(SistemaError::OrdenNoExiste)?;
                self.verificar_permiso_orden(caller, orden_ref, &nuevo_estado)?;
            }
            // Luego pide el borrow mutable para modificar el estado
            let orden = self.obtener_orden_mut(orden_id)?;
            let _estado_anterior = orden.estado.clone();
            orden.estado = nuevo_estado;
            Ok(())
        }






        /// Permite al comprador solicitar la cancelación de una orden en estado `Pendiente`.
        ///
        /// # Qué hace
        /// Registra la intención del comprador de cancelar la orden.  
        /// No ejecuta la cancelación todavía: únicamente marca el campo
        /// `cancelacion_solicitada_por` con la cuenta del comprador.
        ///
        /// # Parámetros
        /// - `orden_id`: Identificador de la orden que se desea cancelar.
        ///
        /// # Retornos
        /// - `Ok(())` si la solicitud se registró correctamente.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::OrdenNoExiste)` si el `orden_id` no corresponde a una orden válida.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no es el comprador de la orden.
        /// - `Err(SistemaError::CancelacionOrdenNoPendiente)` si la orden no está en estado `Pendiente`.
        /// - `Err(SistemaError::CancelacionYaSolicitada)` si ya había una solicitud activa.
        ///
        /// # Notas
        /// - El estado de la orden se mantiene en `Pendiente` hasta que el vendedor acepte.
        /// - Se espera un flujo posterior con `aceptar_cancelacion`.

        #[ink(message)]
        pub fn solicitar_cancelacion(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            self.solicitar_cancelacion_interno(orden_id)
        }
        /// Solicita la cancelación de una orden mientras está **Pendiente**.
        /// - Solo puede invocarla el **Comprador** de la orden.
        /// - La orden debe estar en `EstadoOrden::Pendiente`.
        /// - Si ya existe una solicitud previa, falla.
        /// - No ejecuta la cancelación todavía: solo registra la intención del comprador.
        fn solicitar_cancelacion_interno(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            let caller = self.env().caller();
            self.verificar_registro(caller)?;
            // 1) leer inmutable para validar
            {
                let orden_ref = self.ordenes.get(orden_id as usize).ok_or(SistemaError::OrdenNoExiste)?;
                if orden_ref.estado != EstadoOrden::Pendiente {
                    return Err(SistemaError::CancelacionOrdenNoPendiente); // esta orden no está en estado Pendiente, así que no puede cancelarse
                }
                if caller != orden_ref.comprador {
                    return Err(SistemaError::NoEsRolCorrecto);
                }
                if orden_ref.cancelacion_solicitada_por.is_some() {
                    return Err(SistemaError::CancelacionYaSolicitada);
                }
            }
            // 2) mutar: setear solicitud
            let orden = self.obtener_orden_mut(orden_id)?;
            orden.cancelacion_solicitada_por = Some(orden.comprador);
            Ok(())
        }




        /// Permite al vendedor aceptar una cancelación previamente solicitada.
        ///
        /// # Qué hace
        /// Verifica que la orden tenga una solicitud válida hecha por el comprador
        /// y que siga en estado `Pendiente`. Si todo está correcto:
        /// - Cambia la orden a `Cancelada`.
        /// - Restaura el stock del producto sumando la cantidad de la orden.
        ///
        /// # Parámetros
        /// - `orden_id`: Identificador de la orden a cancelar.
        ///
        /// # Retornos
        /// - `Ok(())` si la cancelación se aplicó.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::OrdenNoExiste)` si el `orden_id` no corresponde a una orden válida.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no es el vendedor de la orden.
        /// - `Err(SistemaError::CancelacionOrdenNoPendiente)` si la orden no está en `Pendiente`.
        /// - `Err(SistemaError::CancelacionNoSolicitada)` si no había solicitud previa.
        ///
        /// # Efectos colaterales
        /// - Marca la orden como `Cancelada`.
        /// - Reintegra la cantidad comprada al stock del producto.
        ///
        /// # Notas
        /// - Solo el vendedor puede aceptar. El comprador nunca puede ejecutar la cancelación por sí mismo.

        #[ink(message)]
        pub fn aceptar_cancelacion(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            self.aceptar_cancelacion_interno(orden_id)
        }
        /// Acepta una cancelación previamente solicitada por el **Comprador**,
        /// siempre que la orden siga en estado **Pendiente**. Al aceptar:
        /// - La orden pasa a `EstadoOrden::Cancelada`.
        /// - (Implementación propuesta) Se **repone** el stock del producto.
        fn aceptar_cancelacion_interno(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            let caller = self.env().caller();
            self.verificar_registro(caller)?;
            // 1) validar con lectura inmutable
            {
                let orden_ref = self.ordenes.get(orden_id as usize).ok_or(SistemaError::OrdenNoExiste)?;
                if orden_ref.estado != EstadoOrden::Pendiente { //Esto es correcto?
                    return Err(SistemaError::CancelacionOrdenNoPendiente);
                }
                if caller != orden_ref.vendedor {
                    return Err(SistemaError::NoEsRolCorrecto);
                }
                match orden_ref.cancelacion_solicitada_por {
                    Some(solicitante) if solicitante == orden_ref.comprador => { /* todo ok, comprador solicitó */ },
                    Some(_) => return Err(SistemaError::NoEsRolCorrecto), // otro actor solicitó (no debería pasar)
                    None => return Err(SistemaError::CancelacionNoSolicitada),
                }
            }
            // 2) mutar: reponer stock y cancelar
            let (producto_id, cantidad) = {
                let o = self.obtener_orden_mut(orden_id)?;
                o.estado = EstadoOrden::Cancelada;
                o.cancelacion_solicitada_por = None;
                (o.producto_id, o.cantidad)
            };
            // reponer stock del producto
            if let Ok(prod) = self.obtener_producto_mut(producto_id) {
                prod.cantidad = prod.cantidad.saturating_add(cantidad);
            }
            Ok(())
        }

        // --- Reputación ---
        /// Califica al vendedor cuando la orden está `Recibida` (solo el comprador).
        ///
        /// # Qué hace
        /// Valida que el caller sea el comprador de la orden, que la orden esté en
        /// `Recibida`, que el `puntaje` ∈ 1..=5 y que no haya calificado antes.
        /// Suma el puntaje a `reputacion_como_vendedor` del vendedor y marca la orden
        /// como ya calificada por el comprador.
        ///
        /// # Parámetros
        /// - `orden_id`: Identificador de la orden recibida.
        /// - `puntaje`: Entero entre 1 y 5.
        ///
        /// # Retornos
        /// - `Ok(())` si se aplicó la calificación.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::OrdenNoExiste)` si `orden_id` es inválido.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no es el comprador.
        /// - `Err(SistemaError::EstadoInvalido)` si la orden no está `Recibida`.
        /// - `Err(SistemaError::PuntajeInvalido)` si `puntaje` no está en 1..=5.
        /// - `Err(SistemaError::OrdenYaCalificada)` si el comprador ya calificó antes.
        ///
        /// # Efectos
        /// - Emite `CompradorCalifico { orden_id, comprador, puntaje }`.
        #[ink(message)]
        pub fn calificar_vendedor(&mut self, orden_id: u32, puntaje: u8) -> Result<(), SistemaError> {
            let caller = self.env().caller();
            self.verificar_registro(caller)?;
            self.verificar_puntaje(puntaje)?;

            // Validación inmutable
            { //obtenemos la orden de forma inmutable para validar
                let o = self.ordenes.get(orden_id as usize).ok_or(SistemaError::OrdenNoExiste)?;
                if o.estado != EstadoOrden::Recibida { 
                    return Err(SistemaError::EstadoInvalido);
                }
                if caller != o.comprador {
                    return Err(SistemaError::NoEsRolCorrecto);
                }
                if o.comprador_califico {
                    return Err(SistemaError::OrdenYaCalificada);
                }
            }

            // Mutación: actualizar reputación y marcar orden como calificada
            // actualiza la orden
            let vendedor = {
                let o = self.obtener_orden_mut(orden_id)?;
                o.comprador_califico = true;
                o.vendedor
            };
            // actualiza la reputación del vendedor
            if let Some(mut u) = self.usuarios.get(&vendedor) {
                u.reputacion_como_vendedor = u.reputacion_como_vendedor.saturating_add(puntaje as u32); // tengo que usar saturating_add para evitar overflow
                self.usuarios.insert(vendedor, &u); // actualizo el usuario en el mapping
            }

            // Emite evento de calificación del comprador al vendedor 
            self.env().emit_event(CompradorCalifico { orden_id, comprador: caller, puntaje });
            Ok(())
        }






        /// Califica al comprador cuando la orden está `Recibida` (solo el vendedor).
        ///
        /// # Qué hace
        /// Valida que el caller sea el vendedor de la orden, que la orden esté en
        /// `Recibida`, que el `puntaje` ∈ 1..=5 y que no haya calificado antes.
        /// Suma el puntaje a `reputacion_como_comprador` del comprador y marca la orden
        /// como ya calificada por el vendedor.
        ///
        /// # Parámetros
        /// - `orden_id`: Identificador de la orden recibida.
        /// - `puntaje`: Entero entre 1 y 5.
        ///
        /// # Retornos
        /// - `Ok(())` si se aplicó la calificación.
        /// - `Err(SistemaError::UsuarioNoRegistrado)` si el caller no está registrado.
        /// - `Err(SistemaError::OrdenNoExiste)` si `orden_id` es inválido.
        /// - `Err(SistemaError::NoEsRolCorrecto)` si el caller no es el vendedor.
        /// - `Err(SistemaError::EstadoInvalido)` si la orden no está `Recibida`.
        /// - `Err(SistemaError::PuntajeInvalido)` si `puntaje` no está en 1..=5.
        /// - `Err(SistemaError::OrdenYaCalificada)` si el vendedor ya calificó antes.
        ///
        /// # Efectos
        /// - Emite `VendedorCalifico { orden_id, vendedor, puntaje }`.
        #[ink(message)]
        pub fn calificar_comprador(&mut self, orden_id: u32, puntaje: u8) -> Result<(), SistemaError> {
            let caller = self.env().caller();
            self.verificar_registro(caller)?;
            self.verificar_puntaje(puntaje)?;

            // Validación inmutable
            {
                let o = self.ordenes.get(orden_id as usize).ok_or(SistemaError::OrdenNoExiste)?;
                if o.estado != EstadoOrden::Recibida {
                    return Err(SistemaError::EstadoInvalido);
                }
                if caller != o.vendedor {
                    return Err(SistemaError::NoEsRolCorrecto);
                }
                if o.vendedor_califico {
                    return Err(SistemaError::OrdenYaCalificada); // ya calificó
                }
            }

            // Mutación: actualizar reputación y marcar orden como calificada
            // actualiza la orden
            let comprador = {
                let o = self.obtener_orden_mut(orden_id)?;
                o.vendedor_califico = true;
                o.comprador
            };
            // actualiza reputación del comprador
            if let Some(mut u) = self.usuarios.get(&comprador) {
                u.reputacion_como_comprador = u.reputacion_como_comprador.saturating_add(puntaje as u32);
                self.usuarios.insert(comprador, &u);
            }

            self.env().emit_event(VendedorCalifico { orden_id, vendedor: caller, puntaje });
            Ok(())
        }




        // --- Funciones de lectura pública ---
        /// Devuelve una copia de la orden si existe.
        #[ink(message)]
        pub fn obtener_orden_pub(&self, orden_id: u32) -> Option<Orden> {
            self.ordenes.get(orden_id as usize).cloned()
        }

        /// Devuelve una copia del producto si existe.
        #[ink(message)]
        pub fn obtener_producto_pub(&self, producto_id: u32) -> Option<Producto> {
            self.productos.iter().find(|p| p.id == producto_id).cloned()
        }

        /// Devuelve las reputaciones acumuladas del usuario, si está registrado.
        /// (rep_como_comprador, rep_como_vendedor)
        #[ink(message)]
        pub fn obtener_reputaciones(&self, usuario: AccountId) -> Option<(u32, u32)> {
            self.usuarios.get(&usuario).map(|u: Usuario| {
                (u.reputacion_como_comprador, u.reputacion_como_vendedor)
            })
        }

        /// Lista copias de todas las órdenes donde `usuario` es comprador o vendedor.
        #[ink(message)]
        pub fn listar_ordenes_por_usuario(&self, usuario: AccountId) -> Vec<Orden> {
            self.ordenes
                .iter()
                .filter(|o| o.comprador == usuario || o.vendedor == usuario)
                .cloned()
                .collect()
        }

        // --- Sistema de disputas (opcional) ---
        // Cómo funciona? funciona así:
        /// Abre una disputa sobre una orden en estado Enviada.
        /// 
        /// Solo puede hacerlo el **Comprador**. Una vez abierta, la orden no puede
        /// ser marcada como recibida o cancelada hasta que el vendedor la resuelva.
        /// 
        /// # Errores
        /// - `UsuarioNoRegistrado` si el caller no está registrado.
        /// - `OrdenNoExiste` si la orden no existe.
        /// - `NoEsRolCorrecto` si el caller no es el comprador.
        /// - `EstadoInvalido` si la orden no está Enviada.
        /// - `DisputaAbierta` si ya existe una disputa activa.
        #[ink(message)]
        pub fn abrir_disputa(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            self.abrir_disputa_interno(orden_id)
        }
        fn abrir_disputa_interno(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            let caller = self.env().caller();
            self.verificar_registro(caller)?;

            let orden = self.obtener_orden_mut(orden_id)?;
            if orden.estado != EstadoOrden::Enviada {
                return Err(SistemaError::EstadoInvalido);
            }
            if caller != orden.comprador {
                return Err(SistemaError::NoEsRolCorrecto);
            }
            if orden.disputa_abierta {
                return Err(SistemaError::DisputaAbierta);
            }

            orden.disputa_abierta = true;
            Ok(())
        }

        /// Resuelve una disputa abierta sobre una orden.
        /// 
        /// Solo puede hacerlo el **Vendedor** de la orden.
        /// Dependiendo del parámetro `a_favor_del_comprador`, la orden pasa a:
        /// - `Cancelada` (si `true`)
        /// - `Recibida` (si `false`)
        /// 
        /// # Errores
        /// - `UsuarioNoRegistrado` si el caller no está registrado.
        /// - `OrdenNoExiste` si la orden no existe.
        /// - `NoEsRolCorrecto` si el caller no es el vendedor.
        /// - `NoHayDisputaAbierta` si la orden no tiene disputa activa.
        #[ink(message)]
        pub fn resolver_disputa(&mut self, orden_id: u32, a_favor_del_comprador: bool) -> Result<(), SistemaError> {
            self.resolver_disputa_interno(orden_id, a_favor_del_comprador)
        }
        fn resolver_disputa_interno(&mut self, orden_id: u32, a_favor_del_comprador: bool) -> Result<(), SistemaError> {
            let caller = self.env().caller();
            self.verificar_registro(caller)?;

            let orden = self.obtener_orden_mut(orden_id)?;
            if !orden.disputa_abierta {
                return Err(SistemaError::NoHayDisputaAbierta);
            }
            if caller != orden.vendedor {
                return Err(SistemaError::NoEsRolCorrecto);
            }

            orden.disputa_abierta = false;
            orden.estado = if a_favor_del_comprador {
                EstadoOrden::Cancelada
            } else {
                EstadoOrden::Recibida
            };

            Ok(())
        }





        // --- Funciones auxiliares ---

        /// Verifica si un usuario está registrado.
        fn verificar_registro(&self, usuario: AccountId) -> Result<(), SistemaError> {
            if !self.usuarios.contains(&usuario) { // Cambia contains_key por contains
                Err(SistemaError::UsuarioNoRegistrado)
            } else {
                Ok(())
            }
        }

        /// Verifica si el usuario tiene el rol requerido.
        fn verificar_rol(&self, usuario: AccountId, rol_requerido: RolUsuario) -> Result<(), SistemaError> {
            let usuario_data = self.usuarios.get(&usuario)
                .ok_or(SistemaError::UsuarioNoRegistrado)?;

            match (usuario_data.rol, rol_requerido) {
                // Solo usuarios con rol Comprador pueden crear órdenes
                (RolUsuario::Comprador, RolUsuario::Comprador) => Ok(()),
                // Solo usuarios con rol Vendedor pueden publicar productos
                (RolUsuario::Vendedor, RolUsuario::Vendedor) => Ok(()),
                // Usuarios con rol Ambos pueden hacer ambas acciones
                (RolUsuario::Ambos, _) => Ok(()),
                _ => Err(SistemaError::NoEsRolCorrecto),
            }
        }

        fn verificar_puede_cambiar_rol(&self, usuario:AccountId, rol_solicitado: RolUsuario) -> Result<(), SistemaError> {
            let usuario_data = self.usuarios.get(&usuario)
                .ok_or(SistemaError::UsuarioNoRegistrado)?;

            match (usuario_data.rol, rol_solicitado) {
                // Solo usuarios con rol Vendedor pueden cambiar a Comprador
                (RolUsuario::Vendedor, RolUsuario::Comprador) => Ok(()),
                // Solo usuarios con rol Comprador pueden cambiar a Vendedor
                (RolUsuario::Comprador, RolUsuario::Vendedor) => Ok(()),
                // Usuarios con rol Ambos pueden cambiar a cualquier rol
                (RolUsuario::Ambos, _) => Ok(()),
                _ => Err(SistemaError::NoEsRolCorrecto),
            }
        }

        /// Verifica específicamente si el usuario puede crear órdenes.
        /// Solo usuarios con rol Comprador o Ambos pueden crear órdenes.
        /// Los usuarios con rol Vendedor no pueden crear órdenes.
        fn verificar_puede_comprar(&self, usuario: AccountId) -> Result<(), SistemaError> {
            let usuario_data = self.usuarios.get(&usuario)
                .ok_or(SistemaError::UsuarioNoRegistrado)?;

            match usuario_data.rol {
                RolUsuario::Comprador | RolUsuario::Ambos => Ok(()),
                RolUsuario::Vendedor => Err(SistemaError::NoEsRolCorrecto),
            }
        }

        /// Verifica que la cantidad sea mayor a cero.
        fn verificar_cantidad(&self, cantidad: u32) -> Result<(), SistemaError> {
            if cantidad == 0 {
                Err(SistemaError::CantidadInsuficiente)
            } else {
                Ok(())
            }
        }

        /// Verifica que hay suficiente stock disponible para la cantidad solicitada.
        fn verificar_stock_disponible(&self, producto: &Producto, cantidad_solicitada: u32) -> Result<(), SistemaError> {
            if producto.cantidad < cantidad_solicitada {
                Err(SistemaError::StockInsuficiente)
            } else {
                Ok(())
            }
        }

        /// Agrega un producto a la lista de productos.
        /// # Nota
        /// Emite un evento `ProductoPublicado` con el `vendedor` y el `producto_id`.   
        fn agregar_producto(
            &mut self,
            nombre: String,
            descripcion: String,
            precio: Balance,
            cantidad: u32,
            categoria: String,
            vendedor: AccountId,
        ) -> Result<(), SistemaError> {
            let id = self.productos.len() as u32;
            let nuevo = Producto::new(id, nombre, descripcion, precio, cantidad, categoria, vendedor);
            self.productos.push(nuevo);

            // Evento de publicación
            self.env().emit_event(ProductoPublicado { vendedor, producto_id: id });

            Ok(())
        }


        /// Obtiene un producto mutable por su id.
        fn obtener_producto_mut(&mut self, id: u32) -> Result<&mut Producto, SistemaError> {
            self.productos
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or(SistemaError::ProductosVacios)
        }

        /// Crea y almacena una nueva orden.
        fn crear_y_emitir_orden(
            &mut self,
            comprador: AccountId,
            vendedor: AccountId,
            producto_id: u32,
            cantidad: u32
        ) -> Result<u32, SistemaError> {
            let id = self.ordenes.len() as u32;
            let nueva_orden = Orden::new(id, comprador, vendedor, producto_id, cantidad);
            self.ordenes.push(nueva_orden.clone());
            // self.emitir_evento_creacion(nueva_orden);
            Ok(id)
        }

        /// Obtiene una orden mutable por su id.
        fn obtener_orden_mut(&mut self, id: u32) -> Result<&mut Orden, SistemaError> {
            self.ordenes
                .get_mut(id as usize)
                .ok_or(SistemaError::OrdenNoExiste)
        }
        /// Verifica si el caller tiene permiso para cambiar el estado de la orden.
        fn verificar_permiso_orden(
            &self,
            caller: AccountId,
            orden: &Orden,
            nuevo_estado: &EstadoOrden
        ) -> Result<(), SistemaError> {
            match nuevo_estado {
                EstadoOrden::Enviada if caller != orden.vendedor => Err(SistemaError::NoEsRolCorrecto),
                EstadoOrden::Recibida if caller != orden.comprador => Err(SistemaError::NoEsRolCorrecto),
                _ => self.verificar_transicion_estado(&orden.estado, nuevo_estado),
            }
        }

        /// Verifica que la transición de estado de la orden sea válida.
        fn verificar_transicion_estado(
            &self,
            actual: &EstadoOrden,
            nuevo: &EstadoOrden
        ) -> Result<(), SistemaError> {
            match (actual, nuevo) {
                (EstadoOrden::Pendiente, EstadoOrden::Enviada) => Ok(()),
                (EstadoOrden::Enviada, EstadoOrden::Recibida) => Ok(()),
                _ => Err(SistemaError::EstadoInvalido),
            }
        }
        
        /// Verifica que el puntaje esté en 1..=5.
        fn verificar_puntaje(&self, puntaje: u8) -> Result<(), SistemaError> {
            if (1..=5).contains(&puntaje) {
                Ok(())
            } else {
                Err(SistemaError::PuntajeInvalido)
            }
        }

        // --- Funciones de listado ---
        /// Lista todos los productos disponibles en el marketplace.
        #[ink(message)]
        pub fn listar_productos_por_categoria(&self, categoria: String) -> Vec<Producto> {
            self.productos.iter().filter(|p| p.categoria == categoria).cloned().collect()
        }

        // Lista todas las órdenes asociadas a un producto específico.
        #[ink(message)]
        pub fn listar_ordenes_por_producto(&self, producto_id: u32) -> Vec<Orden> {
            self.ordenes.iter().filter(|o| o.producto_id == producto_id).cloned().collect()
        }

    } // <-- fin del impl Marketplace

    






    // ────────────────
    // ENUMS
    // ────────────────

    /// Enum para los roles posibles de un usuario.
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum RolUsuario {
        Comprador,
        Vendedor,
        Ambos,
    }

    /// Enum para los posibles estados de una orden.
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum EstadoOrden {
        Pendiente,
        Enviada,
        Recibida,
        Cancelada,
    }

    // ────────────────
    // ERRORES DEL SISTEMA
    // ────────────────

    /// Enum para los posibles errores del sistema.
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]    
    pub enum SistemaError {
        CantidadInsuficiente,
        UsuarioNoRegistrado,
        ProductosVacios,
        NoEsRolCorrecto,
        EstadoInvalido,
        OrdenNoExiste,
        UsuarioExistente,
        StockInsuficiente,
        CancelacionOrdenNoPendiente,
        CancelacionYaSolicitada,
        CancelacionNoSolicitada,
        PuntajeInvalido,
        OrdenYaCalificada,
        DisputaAbierta,
        NoHayDisputaAbierta,
    }
    impl core::fmt::Display for SistemaError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                SistemaError::CantidadInsuficiente => write!(f, "Cantidad insuficiente"),
                SistemaError::UsuarioNoRegistrado => write!(f, "El usuario no se encuentra registrado"),
                SistemaError::NoEsRolCorrecto => write!(f, "El usuario no es del rol correcto"),
                SistemaError::ProductosVacios => write!(f, "No se encontraron productos"),
                SistemaError::EstadoInvalido => write!(f, "El estado de la orden es inválido"),
                SistemaError::OrdenNoExiste => write!(f, "La orden no existe"),
                SistemaError::UsuarioExistente => write!(f, "El usuario ya está registrado"),
                SistemaError::StockInsuficiente => write!(f, "Stock insuficiente para la cantidad solicitada"),
                SistemaError::CancelacionOrdenNoPendiente => write!(f, "La orden no está en estado pendiente"),
                SistemaError::CancelacionYaSolicitada => write!(f, "La cancelación ya fue solicitada"),
                SistemaError::CancelacionNoSolicitada => write!(f, "La cancelación no ha sido solicitada"),
                SistemaError::PuntajeInvalido => write!(f, "El puntaje debe estar entre 1 y 5"),
                SistemaError::OrdenYaCalificada => write!(f, "La orden ya ha sido calificada"),
                SistemaError::DisputaAbierta => write!(f, "Ya existe una disputa abierta para esta orden"),
                SistemaError::NoHayDisputaAbierta => write!(f, "No hay una disputa abierta para esta orden"),
            }
        }
    }

    // ────────────────
    // ESTRUCTURAS PRINCIPALES
    // ────────────────

    /// Representa un usuario del marketplace.
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]    
    pub struct Usuario {
        /// Dirección de la cuenta del usuario.
        pub direccion: AccountId,
        /// Rol asignado al usuario.
        pub rol: RolUsuario,
        /// Reputación como comprador.
        pub reputacion_como_comprador: u32,
        /// Reputación como vendedor.
        pub reputacion_como_vendedor: u32,
    }

    /// Representa un producto publicado en el marketplace.
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]    
    pub struct Producto {
        /// Identificador único del producto.
        pub id: u32,
        /// Nombre del producto.
        pub nombre: String,
        /// Descripción del producto.
        pub descripcion: String,
        /// Precio del producto.
        pub precio: Balance,
        /// Cantidad disponible.
        pub cantidad: u32,
        /// Categoría del producto.
        pub categoria: String,
        /// Dirección del vendedor.
        pub vendedor: AccountId,
    }
    impl Producto {
        /// Crea una nueva instancia de Producto.
        pub fn new(id: u32, nombre: String, descripcion: String, precio: Balance, cantidad: u32, categoria: String, vendedor: AccountId) -> Self {
            Self {
                id,
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
                vendedor,
            }
        }
    }

    /// Representa una orden de compra.
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]    
    pub struct Orden {
        /// Identificador único de la orden.
        pub id: u32,
        /// Dirección del comprador.
        pub comprador: AccountId,
        /// Dirección del vendedor.
        pub vendedor: AccountId,
        /// Identificador del producto comprado.
        pub producto_id: u32,
        /// Cantidad comprada.
        pub cantidad: u32,
        /// Estado actual de la orden.
        pub estado: EstadoOrden,
        /// Indica si el comprador calificó.
        pub comprador_califico: bool,
        /// Indica si el vendedor calificó.
        pub vendedor_califico: bool,
        /// Indica quien pidió cancelar (solo permitido si está pendiente)
        pub cancelacion_solicitada_por: Option<AccountId>,
        /// Indica si existe una disputa abierta sobre la orden
        pub disputa_abierta: bool,
    }
    impl Orden {
        /// Crea una nueva instancia de Orden.
        pub fn new(id: u32, comprador: AccountId, vendedor: AccountId, producto_id: u32, cantidad: u32) -> Self {
            Self {
                id,
                comprador,
                vendedor,
                producto_id,
                cantidad,
                estado: EstadoOrden::Pendiente,
                comprador_califico: false,
                vendedor_califico: false,
                cancelacion_solicitada_por: None,
                disputa_abierta: false,
            }
        }
    }

    // ────────────────
    // EVENTOS
    // ────────────────

    #[ink(event)]
    pub struct RolActualizado {
        #[ink(topic)]
        cuenta: AccountId,
        rol_anterior: RolUsuario,
        rol_nuevo: RolUsuario,
    }

    #[ink(event)]
    pub struct ProductoPublicado {
        #[ink(topic)]
        vendedor: AccountId,
        #[ink(topic)]
        producto_id: u32,
    }

    #[ink(event)]
    pub struct CompradorCalifico {
        #[ink(topic)]
        orden_id: u32,
        #[ink(topic)]
        comprador: AccountId,
        puntaje: u8,
    }

    #[ink(event)]
    pub struct VendedorCalifico {
        #[ink(topic)]
        orden_id: u32,
        #[ink(topic)]
        vendedor: AccountId,
        puntaje: u8,
    }













    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        // Vendedor = alice
        fn setup_contract_con_vendedor() -> MarketplacePrincipal {
            let mut contrato = MarketplacePrincipal::new();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            let usuario = Usuario {
                direccion: acc.alice,
                rol: RolUsuario::Vendedor,
                reputacion_como_comprador: 0,
                reputacion_como_vendedor: 0,
            };
            contrato.usuarios.insert(acc.alice, &usuario);
            contrato
        }

        // Comprador = bob
        fn setup_contract_con_comprador() -> MarketplacePrincipal {
            let mut contrato = MarketplacePrincipal::new();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            let usuario = Usuario {
                direccion: acc.bob,
                rol: RolUsuario::Comprador,
                reputacion_como_comprador: 0,
                reputacion_como_vendedor: 0,
            };
            contrato.usuarios.insert(acc.bob, &usuario);
            contrato
        }

        
        // --- Registro de usuarios ---
        #[ink::test]
        fn registrar_usuario_comprador_ok() {
            let mut contrato = MarketplacePrincipal::new();

            // Simulamos que el caller es "Alice"
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            // Llamamos a la función registrar_usuario con el rol de comprador
            let resultado = contrato.registrar_usuario(RolUsuario::Comprador);

            // Verificamos que devuelva OK
            assert_eq!(resultado, Ok(()));

            // Obtenemos el usuario usando la dirección del caller
            let usuario_registrado = contrato.usuarios.get(&accounts.alice);

            // Confirmamos si se guardó el usuario
            assert!(usuario_registrado.is_some());

            // Verificamos los datos
            let usuario = usuario_registrado.unwrap();
            assert_eq!(usuario.rol, RolUsuario::Comprador);
            assert_eq!(usuario.reputacion_como_comprador, 0);
            assert_eq!(usuario.reputacion_como_vendedor, 0);
        }

        #[ink::test]
        fn registrar_usuario_vendedor_ok() {
            let mut contrato = MarketplacePrincipal::new();

            // Simulamos que el caller es "Bob"
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            // Llamamos a la función registrar_usuario con el rol de vendedor
            let resultado = contrato.registrar_usuario(RolUsuario::Vendedor);

            // Verificamos que devuelva OK
            assert_eq!(resultado, Ok(()));

            // Obtenemos el usuario usando la dirección del caller
            let usuario_registrado = contrato.usuarios.get(&accounts.bob);

            // Confirmamos si se guardó el usuario
            assert!(usuario_registrado.is_some());

            // Verificamos los datos
            let usuario = usuario_registrado.unwrap();
            assert_eq!(usuario.rol, RolUsuario::Vendedor);
            assert_eq!(usuario.reputacion_como_comprador, 0);
            assert_eq!(usuario.reputacion_como_vendedor, 0);
        }

        #[ink::test]
        fn registrar_usuario_ambos_ok() {
            let mut contrato = MarketplacePrincipal::new();

            // Simulamos que el caller es "Charlie"
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);

            // Llamamos a la función registrar_usuario con el rol de ambos
            let resultado = contrato.registrar_usuario(RolUsuario::Ambos);

            // Verificamos que devuelva OK
            assert_eq!(resultado, Ok(()));

            // Obtenemos el usuario usando la dirección del caller
            let usuario_registrado = contrato.usuarios.get(&accounts.charlie);

            // Confirmamos si se guardó el usuario
            assert!(usuario_registrado.is_some());

            // Verificamos los datos
            let usuario = usuario_registrado.unwrap();
            assert_eq!(usuario.rol, RolUsuario::Ambos);
            assert_eq!(usuario.reputacion_como_comprador, 0);
            assert_eq!(usuario.reputacion_como_vendedor, 0);
        }

        #[ink::test]
        fn registrar_usuario_duplicado_falla() {
            let mut contrato = MarketplacePrincipal::new();

            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            // Primer registro
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Segundo registro debería fallar porque ya está registrado
            let resultado = contrato.registrar_usuario(RolUsuario::Vendedor);
            assert_eq!(resultado, Err(SistemaError::UsuarioExistente));
        }

        // --- Modificación de roles ---
        #[ink::test]
        fn modificar_rol_usuario_comprador_a_vendedor_ok() {
            let mut contrato = setup_contract_con_vendedor(); // alice = vendedor
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            contrato.registrar_usuario(RolUsuario::Comprador).unwrap();

            let r = contrato.modificar_rol_usuario(RolUsuario::Vendedor);
            assert!(r.is_ok());
            assert_eq!(contrato.obtener_usuario(acc.bob).unwrap().rol, RolUsuario::Vendedor);
        }

        #[ink::test]
        fn modificar_rol_usuario_vendedor_a_comprador_ok() {
            let mut contrato = setup_contract_con_vendedor(); // alice = vendedor
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // usar charlie como vendedor propio para no interferir con alice
            test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            contrato.registrar_usuario(RolUsuario::Vendedor).unwrap();

            let r = contrato.modificar_rol_usuario(RolUsuario::Comprador);
            assert!(r.is_ok());
            assert_eq!(contrato.obtener_usuario(acc.charlie).unwrap().rol, RolUsuario::Comprador);
        }

        #[ink::test]
        fn modificar_rol_usuario_ambos_a_comprador_ok() {
            let mut contrato = setup_contract_con_vendedor();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            contrato.registrar_usuario(RolUsuario::Ambos).unwrap();

            let r = contrato.modificar_rol_usuario(RolUsuario::Comprador);
            assert!(r.is_ok());
            assert_eq!(contrato.obtener_usuario(acc.charlie).unwrap().rol, RolUsuario::Comprador);
        }

        #[ink::test]
        fn modificar_rol_usuario_ambos_a_vendedor_ok() {
            let mut contrato = setup_contract_con_vendedor();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Registramos a Charlie como Ambos
            test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            contrato.registrar_usuario(RolUsuario::Ambos).unwrap();

            // Cambiamos su rol a Vendedor
            let r = contrato.modificar_rol_usuario(RolUsuario::Vendedor);
            assert!(r.is_ok());

            // Verificamos que el cambio se aplicó correctamente
            let usuario = contrato.obtener_usuario(acc.charlie).unwrap();
            assert_eq!(usuario.rol, RolUsuario::Vendedor);
        }


        #[ink::test]
        fn modificar_rol_usuario_mismo_rol_falla() {
            let mut contrato = setup_contract_con_vendedor();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            contrato.registrar_usuario(RolUsuario::Comprador).unwrap();

            let r = contrato.modificar_rol_usuario(RolUsuario::Comprador);
            assert!(matches!(r, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn modificar_rol_usuario_no_registrado_falla() {
            let mut contrato = MarketplacePrincipal::new();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // caller no registrado
            test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            let r = contrato.modificar_rol_usuario(RolUsuario::Vendedor);
            assert!(matches!(r, Err(SistemaError::UsuarioNoRegistrado)));
        }



        #[ink::test]
        fn emite_evento_rol_actualizado() {
            let mut c = MarketplacePrincipal::new();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);

            c.registrar_usuario(RolUsuario::Comprador).unwrap();

            // Ejecutamos la acción que debería emitir el evento
            c.modificar_rol_usuario(RolUsuario::Vendedor).unwrap();

            // Recolectamos todos los eventos emitidos hasta ahora
            let eventos = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert!(!eventos.is_empty(), "Debe emitirse al menos un evento");
        }


        // --- Publicación de productos ---
        #[ink::test]
        fn publicar_producto_ok() {
            let mut contrato = setup_contract_con_vendedor();

            let resultado = contrato.publicar_producto(
                "Celular".to_string(),
                "Un buen celular".to_string(),
                1000,
                5,
                "Tecnología".to_string(),
            );

            assert!(resultado.is_ok());
            assert_eq!(contrato.productos.len(), 1);

            let producto = &contrato.productos[0];
            assert_eq!(producto.nombre, "Celular");
            assert_eq!(producto.precio, 1000);
        }

        #[ink::test]
        fn publicar_producto_no_registrado_falla() {
            let mut contrato = MarketplacePrincipal::new();

            let caller = AccountId::from([0x02; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(caller);

            let resultado = contrato.publicar_producto(
                "Producto".to_string(),
                "Sin registro".to_string(),
                500,
                1,
                "Otros".to_string(),
            );

            assert!(matches!(resultado, Err(SistemaError::UsuarioNoRegistrado)));
        }

        #[ink::test]
        fn publicar_producto_no_es_vendedor_falla() {
            let mut contrato = MarketplacePrincipal::new();

            let caller = AccountId::from([0x03; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(caller);

            let usuario = Usuario {
                direccion: caller,
                rol: RolUsuario::Comprador, // Rol no válido para publicar productos
                reputacion_como_comprador: 0,
                reputacion_como_vendedor: 0,
            };
            contrato.usuarios.insert(caller, &usuario);

            let resultado = contrato.publicar_producto(
                "Producto".to_string(),
                "No autorizado".to_string(),
                100,
                2,
                "Otros".to_string(),
            );

            assert!(matches!(resultado, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn publicar_producto_cantidad_cero_falla() {
            let mut contrato = setup_contract_con_vendedor();

            let resultado = contrato.publicar_producto(
                "Producto".to_string(),
                "Cantidad cero".to_string(),
                100,
                0, // Cantidad inválida
                "Otros".to_string(),
            );

            assert!(matches!(resultado, Err(SistemaError::CantidadInsuficiente)));
        }

        #[ink::test]
        fn emite_evento_producto_publicado_con_campos_correctos() {
            use parity_scale_codec::Decode;

            let mut c = setup_contract_con_vendedor();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P1".into(), "D".into(), 100, 5, "Cat".into()).unwrap();

            let eventos: Vec<_> = ink::env::test::recorded_events().collect();
            assert!(!eventos.is_empty(), "Debe emitirse ProductoPublicado");

            let ultimo = eventos.last().unwrap();
            let mut bytes = ultimo.data.as_slice();
            let e: ProductoPublicado = Decode::decode(&mut bytes).expect("decode evento");
            assert_eq!(e.producto_id, 0);
            assert_eq!(e.vendedor, acc.alice); // vendedor del helper
        }




        // --- Listar productos ---
        #[ink::test]
        fn listar_interno_ok_para_vendedor() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P1".into(), "D".into(), 100, 5, "Cat".into()).unwrap();
            c.publicar_producto("P2".into(), "D".into(), 200, 3, "Cat".into()).unwrap();

            let v = c.listar_productos_interno(acc.alice).unwrap();
            assert_eq!(v.len(), 2);
            assert!(v.iter().all(|p| p.vendedor == acc.alice));
        }

        /// Error: usuario no registrado intenta listar.
        #[ink::test]
        fn listar_interno_falla_si_no_registrado() {
            let c = MarketplacePrincipal::new();
            let no_reg = AccountId::from([9u8; 32]);

            let res = c.listar_productos_interno(no_reg);
            assert!(matches!(res, Err(SistemaError::UsuarioNoRegistrado)));
        }

        /// Error: registrado como Comprador (no Vendedor/Ambos) intenta listar.
        #[ink::test]
        fn listar_interno_falla_si_no_es_vendedor() {
            let mut c = MarketplacePrincipal::new();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();

            let res = c.listar_productos_interno(acc.bob);
            assert!(matches!(res, Err(SistemaError::NoEsRolCorrecto)));
        }

        /// Error: vendedor válido pero sin productos publicados.
        #[ink::test]
        fn listar_interno_falla_si_no_tiene_productos() {
            let c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            let res = c.listar_productos_interno(acc.alice);
            assert!(matches!(res, Err(SistemaError::ProductosVacios)));
        }

        // --- Listar productos por vendedor ---
        #[ink::test]
        fn listar_productos_por_vendedor_ok() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P1".into(), "D".into(), 100, 5, "Cat".into()).unwrap();
            c.publicar_producto("P2".into(), "D".into(), 200, 3, "Cat".into()).unwrap();

            let productos = c.listar_productos_por_vendedor(acc.alice).unwrap();
            assert_eq!(productos.len(), 2);
            assert_eq!(productos[0].nombre, "P1");
            assert_eq!(productos[1].nombre, "P2");
        }

        #[ink::test]
        fn listar_productos_por_vendedor_vacio_falla() {
            let c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            let res = c.listar_productos_por_vendedor(acc.alice);
            assert!(matches!(res, Err(SistemaError::ProductosVacios)));
        }







        // --- Compra y órdenes ---
        #[ink::test]
        fn crear_orden_ok() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto y obtiene el ID
            let _ = contrato.publicar_producto(
                "Laptop".to_string(),
                "Una laptop potente".to_string(),
                2000,
                10,
                "Tecnología".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // El producto publicado tendrá id = 0 (si es el primero)
            let resultado = contrato.crear_orden(0, 2);

            assert!(resultado.is_ok());
            let orden_id = resultado.unwrap();
            assert_eq!(contrato.ordenes.len(), 1);

            let orden = &contrato.ordenes[0];
            assert_eq!(orden.id, orden_id);
            assert_eq!(orden.cantidad, 2);
            assert_eq!(orden.estado, EstadoOrden::Pendiente);
        }

        #[ink::test]
        fn crear_orden_no_registrado_falla() {
            let mut contrato = MarketplacePrincipal::new();

            let caller = AccountId::from([0x04; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(caller);

            let resultado = contrato.crear_orden(0, 1);

            assert!(matches!(resultado, Err(SistemaError::UsuarioNoRegistrado)));
        }

        #[ink::test]
        fn verificar_registro_antes_de_crear_orden() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario NO registrado
            let nuevo_usuario = AccountId::from([0x99; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(nuevo_usuario);

            // Verifica que el usuario no está registrado
            assert!(!contrato.esta_registrado(nuevo_usuario));
            assert!(contrato.obtener_usuario(nuevo_usuario).is_none());

            // Intenta crear una orden y falla porque no está registrado
            let resultado = contrato.crear_orden(0, 1);
            assert!(matches!(resultado, Err(SistemaError::UsuarioNoRegistrado)));

            // Registra al usuario como comprador
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Verifica que ahora está registrado
            assert!(contrato.esta_registrado(nuevo_usuario));
            let usuario_info = contrato.obtener_usuario(nuevo_usuario).unwrap();
            assert_eq!(usuario_info.rol, RolUsuario::Comprador);

            // Ahora puede crear una orden exitosamente
            let resultado = contrato.crear_orden(0, 1);
            assert!(resultado.is_ok());
        }

        #[ink::test]
        fn crear_orden_no_es_comprador_falla() {
            let mut contrato = MarketplacePrincipal::new();

            let caller = AccountId::from([0x05; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(caller);

            let usuario = Usuario {
                direccion: caller,
                rol: RolUsuario::Vendedor, // Rol no válido para crear órdenes
                reputacion_como_comprador: 0,
                reputacion_como_vendedor: 0,
            };
            contrato.usuarios.insert(caller, &usuario);

            // Primero, publica un producto para poder comprarlo
            let _ = contrato.publicar_producto(
                "Tablet".to_string(),
                "Una tablet versátil".to_string(),
                1500,
                7,
                "Tecnología".to_string(),
            );

            let resultado = contrato.crear_orden(0, 1);

            assert!(matches!(resultado, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn crear_orden_con_rol_ambos_ok() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario con rol Ambos
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            let _ = contrato.registrar_usuario(RolUsuario::Ambos);

            // Debería poder crear una orden exitosamente
            let resultado = contrato.crear_orden(0, 1);
            assert!(resultado.is_ok());
        }

        #[ink::test]
        fn crear_orden_cantidad_insuficiente_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Primero, publica un producto con cantidad insuficiente
            let _ = contrato.publicar_producto(
                "Smartwatch".to_string(),
                "Un smartwatch elegante".to_string(),
                500,
                2, // Solo hay 2 disponibles
                "Tecnología".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Intenta crear una orden de compra de 3 unidades
            let resultado = contrato.crear_orden(0, 3); // Compra 3 unidades

            assert!(matches!(resultado, Err(SistemaError::StockInsuficiente)));
        }

        #[ink::test]
        fn crear_orden_cantidad_cero_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Intenta crear una orden con cantidad 0
            let resultado = contrato.crear_orden(0, 0);

            assert!(matches!(resultado, Err(SistemaError::CantidadInsuficiente)));
        }

        #[ink::test]
        fn crear_orden_descuenta_stock() {
            let mut contrato = setup_contract_con_vendedor();

            // Primero, publica un producto con cantidad suficiente
            let _ = contrato.publicar_producto(
                "Auriculares".to_string(),
                "Auriculares inalámbricos".to_string(),
                800,
                10, // 10 disponibles
                "Tecnología".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Crea una orden de compra
            let resultado = contrato.crear_orden(0, 3); // Compra 3 unidades

            assert!(resultado.is_ok());
            let _orden_id = resultado.unwrap();
            assert_eq!(contrato.ordenes.len(), 1);

            // Verifica que el stock se haya descontado correctamente
            let producto = &contrato.productos[0];
            assert_eq!(producto.cantidad, 7); // Debería quedar 7 después de la compra
        }

        

        // --- Gestión de órdenes ---
        #[ink::test]
        fn marcar_orden_como_enviada_ok() {
            let mut c = setup_contract_con_vendedor();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publica y crea orden: comprador = Bob, vendedor = Alice(0x01... helper)
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Vendedor marca enviada
            let vendedor = AccountId::from([0x01; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
            let r = c.marcar_orden_como_enviada(id);
            assert!(r.is_ok());
            assert_eq!(c.ordenes[id as usize].estado, EstadoOrden::Enviada);
        }

        #[ink::test]
        fn marcar_orden_como_enviada_usuario_incorrecto_falla() {
            let mut c = setup_contract_con_vendedor();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publica y crea orden
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Otro usuario (no vendedor de la orden) intenta marcar enviada
            test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            c.registrar_usuario(RolUsuario::Vendedor).unwrap();
            let r = c.marcar_orden_como_enviada(id);
            assert!(matches!(r, Err(SistemaError::NoEsRolCorrecto)));
        }


        #[ink::test]
        fn marcar_como_recibida_usuario_incorrecto_falla() {
            let mut c = setup_contract_con_vendedor();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publica y crea orden
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Vendedor marca enviada
            let vendedor = AccountId::from([0x01; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
            c.marcar_orden_como_enviada(id).unwrap();

            // Un tercero (no comprador) intenta marcar recibida
            test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let r = c.marcar_como_recibida(id);
            assert!(matches!(r, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn transicion_estado_invalida_falla() {
            let mut c = setup_contract_con_vendedor();
            let acc = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publica y crea orden
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Comprador intenta marcar recibida directamente desde PENDIENTE
            // (según verificar_permiso_orden, primero falla por EstadoInvalido)
            let r = c.marcar_como_recibida(id);
            assert!(matches!(r, Err(SistemaError::EstadoInvalido)));
        }


        // --- Errores y validaciones ---
        #[ink::test]
        fn acceder_orden_inexistente_falla() {
            let mut contrato = MarketplacePrincipal::new();

            let resultado = contrato.obtener_orden_mut(999); // ID que no existe

            assert!(matches!(resultado, Err(SistemaError::OrdenNoExiste)));
        }

        #[ink::test]
        fn acceder_producto_inexistente_falla() {
            let mut contrato = MarketplacePrincipal::new();

            let resultado = contrato.obtener_producto_mut(999); // ID que no existe

            assert!(matches!(resultado, Err(SistemaError::ProductosVacios)));
        }

        #[ink::test]
        fn marcar_orden_como_enviada_usuario_no_registrado_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Crea una orden
            let resultado = contrato.crear_orden(0, 1);
            assert!(resultado.is_ok());
            let orden_id = resultado.unwrap();

            // Cambia el caller a un usuario NO registrado
            let usuario_no_registrado = AccountId::from([0x99; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(usuario_no_registrado);

            // Intenta marcar la orden como enviada
            let resultado = contrato.marcar_orden_como_enviada(orden_id);
            assert!(matches!(resultado, Err(SistemaError::UsuarioNoRegistrado)));
        }

        #[ink::test]
        fn marcar_como_recibida_usuario_no_registrado_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Crea una orden
            let resultado = contrato.crear_orden(0, 1);
            assert!(resultado.is_ok());
            let orden_id = resultado.unwrap();

            // Cambia el caller a un usuario NO registrado
            let usuario_no_registrado = AccountId::from([0x99; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(usuario_no_registrado);

            // Intenta marcar la orden como recibida
            let resultado = contrato.marcar_como_recibida(orden_id);
            assert!(matches!(resultado, Err(SistemaError::UsuarioNoRegistrado)));
        }

        #[ink::test]
        fn marcar_orden_como_enviada_usuario_no_es_vendedor_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Crea una orden
            let resultado = contrato.crear_orden(0, 1);
            assert!(resultado.is_ok());
            let orden_id = resultado.unwrap();

            // Cambia el caller a otro usuario registrado que NO es el vendedor
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            let _ = contrato.registrar_usuario(RolUsuario::Vendedor);

            // Intenta marcar la orden como enviada (no debería poder porque no es el vendedor de esta orden)
            let resultado = contrato.marcar_orden_como_enviada(orden_id);
            assert!(matches!(resultado, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn marcar_como_recibida_usuario_no_es_comprador_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Crea una orden
            let resultado = contrato.crear_orden(0, 1);
            assert!(resultado.is_ok());
            let orden_id = resultado.unwrap();

            // Cambia el caller de vuelta al vendedor para marcar como enviada
            let vendedor = AccountId::from([0x01; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(vendedor);

            // Marca la orden como enviada
            let resultado = contrato.marcar_orden_como_enviada(orden_id);
            assert!(resultado.is_ok());

            // Cambia el caller a otro usuario registrado que NO es el comprador
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Intenta marcar la orden como recibida (no debería poder porque no es el comprador de esta orden)
            let resultado = contrato.marcar_como_recibida(orden_id);
            assert!(matches!(resultado, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn marcar_orden_como_enviada_orden_inexistente_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Intenta marcar una orden inexistente como enviada
            let resultado = contrato.marcar_orden_como_enviada(999);
            assert!(matches!(resultado, Err(SistemaError::OrdenNoExiste)));
        }

        #[ink::test]
        fn marcar_como_recibida_orden_inexistente_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Intenta marcar una orden inexistente como recibida
            let resultado = contrato.marcar_como_recibida(999);
            assert!(matches!(resultado, Err(SistemaError::OrdenNoExiste)));
        }

        #[ink::test]
        fn marcar_como_recibida_ok() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Crea una orden
            let resultado = contrato.crear_orden(0, 1);
            assert!(resultado.is_ok());
            let orden_id = resultado.unwrap();

            // Cambia el caller de vuelta al vendedor para marcar como enviada
            let vendedor = AccountId::from([0x01; 32]);
            test::set_caller::<ink::env::DefaultEnvironment>(vendedor);

            // Marca la orden como enviada
            let resultado = contrato.marcar_orden_como_enviada(orden_id);
            assert!(resultado.is_ok());

            // Cambia el caller de vuelta al comprador para marcar como recibida
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            // Marca la orden como recibida (debe ser exitoso)
            let resultado = contrato.marcar_como_recibida(orden_id);
            assert!(resultado.is_ok());

            // Verifica que el estado cambió a Recibida
            let orden = &contrato.ordenes[orden_id as usize];
            assert_eq!(orden.estado, EstadoOrden::Recibida);
        }

        #[ink::test]
        fn marcar_como_recibida_estado_pendiente_falla() {
            let mut contrato = setup_contract_con_vendedor();

            // Publica un producto
            let _ = contrato.publicar_producto(
                "Producto Test".to_string(),
                "Descripción Test".to_string(),
                1000,
                10,
                "Test".to_string(),
            );

            // Cambia el caller a un usuario comprador y regístralo
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            // Crea una orden (estado inicial: Pendiente)
            let resultado = contrato.crear_orden(0, 1);
            assert!(resultado.is_ok());
            let orden_id = resultado.unwrap();

            // Verifica que la orden está en estado Pendiente
            let orden = &contrato.ordenes[orden_id as usize];
            assert_eq!(orden.estado, EstadoOrden::Pendiente);

            // Intenta marcar la orden como recibida directamente desde Pendiente (debe fallar)
            let resultado = contrato.marcar_como_recibida(orden_id);
            assert!(matches!(resultado, Err(SistemaError::EstadoInvalido)));
        }

        
        // -- Cancelación de ordenes
        #[ink::test]
        fn cancelar_pendiente_exitoso_repone_stock() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar producto (alice=vendedora)
            c.publicar_producto("P".into(), "D".into(), 100, 3, "Cat".into()).unwrap();

            // Registrar comprador y crear orden x2
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 2).unwrap();

            // Stock quedó 1
            assert_eq!(c.productos[0].cantidad, 1);

            // Comprador solicita cancelación
            let r = c.solicitar_cancelacion(id);
            assert!(r.is_ok());

            // Vendedor acepta
            let vendedor = acc.alice;
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
            let r2 = c.aceptar_cancelacion(id);
            assert!(r2.is_ok());

            // Orden Cancelada y stock repuesto (1 + 2 = 3)
            assert_eq!(c.ordenes[id as usize].estado, EstadoOrden::Cancelada);
            assert_eq!(c.productos[0].cantidad, 3);
        }

        #[ink::test]
        fn solicitar_cancelacion_falla_si_no_pendiente() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();

            // Bob compra 1
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Vendedor marca enviada → ya no está Pendiente
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(id).unwrap();

            // Bob intenta solicitar cancelación (debe fallar)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            let r = c.solicitar_cancelacion(id);
            assert!(matches!(r, Err(SistemaError::CancelacionOrdenNoPendiente)));
        }

        #[ink::test]
        fn aceptar_cancelacion_falla_si_no_pendiente() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 2, "Cat".into()).unwrap();

            // Bob compra 1 (Pendiente)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Bob solicita cancelación OK
            assert!(c.solicitar_cancelacion(id).is_ok());

            // Vendedor marca enviada (cambia a Enviada)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(id).unwrap();

            // Ahora aceptar_cancelacion debe fallar por no estar Pendiente
            let r = c.aceptar_cancelacion(id);
            assert!(matches!(r, Err(SistemaError::CancelacionOrdenNoPendiente)));
        }

        #[ink::test]
        fn aceptar_cancelacion_sin_solicitud_falla() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();

            // Bob compra 1 (Pendiente)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Vendedor intenta aceptar sin solicitud (falla)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            let r = c.aceptar_cancelacion(id);
            assert!(matches!(r, Err(SistemaError::CancelacionNoSolicitada)));
        }

        #[ink::test]
        fn solicitar_cancelacion_doble_falla() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();

            // Bob compra 1
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Solicita una vez
            assert!(c.solicitar_cancelacion(id).is_ok());
            // Vuelve a solicitar (debe fallar)
            let r = c.solicitar_cancelacion(id);
            assert!(matches!(r, Err(SistemaError::CancelacionYaSolicitada)));
        }

        #[ink::test]
        fn actores_incorrectos_no_pueden_cancelar() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();

            // Bob compra 1
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Un tercero intenta solicitar (no es comprador)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let r = c.solicitar_cancelacion(id);
            assert!(matches!(r, Err(SistemaError::NoEsRolCorrecto)));

            // Vuelve Bob, solicita OK
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            assert!(c.solicitar_cancelacion(id).is_ok());

            // Un tercero intenta aceptar (no es vendedor)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            let r2 = c.aceptar_cancelacion(id);
            assert!(matches!(r2, Err(SistemaError::NoEsRolCorrecto)));
        }


        // ─────────────────────────────────────────────────────────────
        // REPUTACIÓN: tests de calificar_vendedor / calificar_comprador
        // ─────────────────────────────────────────────────────────────

        #[ink::test]
        fn calificar_vendedor_funciona() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar y generar orden
            c.publicar_producto("P".into(), "D".into(), 100, 2, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Enviar → Recibir
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(id).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.marcar_como_recibida(id).unwrap();

            // Calificar vendedor (caller = comprador)
            let r = c.calificar_vendedor(id, 5);
            assert!(r.is_ok());

            // Flag seteado
            assert!(c.ordenes[id as usize].comprador_califico);

            // Reputación del vendedor subió 5
            let rep_v = c.obtener_reputaciones(acc.alice).unwrap().1;
            assert_eq!(rep_v, 5);
        }

        #[ink::test]
        fn calificar_vendedor_falla_si_no_recibida() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Aún está Pendiente → debe fallar
            let r = c.calificar_vendedor(id, 4);
            assert!(matches!(r, Err(SistemaError::EstadoInvalido)));

            // Enviada pero no Recibida → sigue fallando
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(id).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            let r2 = c.calificar_vendedor(id, 4);
            assert!(matches!(r2, Err(SistemaError::EstadoInvalido)));
        }

        #[ink::test]
        fn calificar_vendedor_falla_si_caller_no_es_comprador() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(id).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.marcar_como_recibida(id).unwrap();

            // Charlie intenta calificar vendedor (no es comprador de la orden)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let r = c.calificar_vendedor(id, 3);
            assert!(matches!(r, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn calificar_vendedor_falla_puntaje_invalido_y_doble_calificacion() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(id).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.marcar_como_recibida(id).unwrap();

            // Puntajes inválidos → PuntajeInvalido
            assert!(matches!(c.calificar_vendedor(id, 0), Err(SistemaError::PuntajeInvalido)));
            assert!(matches!(c.calificar_vendedor(id, 6), Err(SistemaError::PuntajeInvalido)));

            // Calificación válida
            assert!(c.calificar_vendedor(id, 4).is_ok());
            // Doble calificación del mismo actor → OrdenYaCalificada
            assert!(matches!(c.calificar_vendedor(id, 5), Err(SistemaError::OrdenYaCalificada)));
        }

        #[ink::test]
        fn calificar_comprador_funciona() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 2, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Recibida
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(id).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.marcar_como_recibida(id).unwrap();

            // Califica vendedor→comprador (caller = vendedor)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            let r = c.calificar_comprador(id, 5);
            assert!(r.is_ok());

            // Flag
            assert!(c.ordenes[id as usize].vendedor_califico);

            // Reputación del comprador subió 5
            let rep_c = c.obtener_reputaciones(acc.bob).unwrap().0;
            assert_eq!(rep_c, 5);
        }

        #[ink::test]
        fn calificar_comprador_fallas_varias() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id = c.crear_orden(0, 1).unwrap();

            // Aún no recibida → EstadoInvalido
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            assert!(matches!(c.calificar_comprador(id, 4), Err(SistemaError::EstadoInvalido)));

            // Avanza a Recibida
            c.marcar_orden_como_enviada(id).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.marcar_como_recibida(id).unwrap();

            // Caller incorrecto (comprador intentando calificar comprador) → NoEsRolCorrecto
            assert!(matches!(c.calificar_comprador(id, 4), Err(SistemaError::NoEsRolCorrecto)));

            // Puntajes inválidos → PuntajeInvalido
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            assert!(matches!(c.calificar_comprador(id, 0), Err(SistemaError::PuntajeInvalido)));
            assert!(matches!(c.calificar_comprador(id, 9), Err(SistemaError::PuntajeInvalido)));

            // Califica 3 OK y reintenta → OrdenYaCalificada
            assert!(c.calificar_comprador(id, 3).is_ok());
            assert!(matches!(c.calificar_comprador(id, 5), Err(SistemaError::OrdenYaCalificada)));
        }

        // ─────────────────────────────────────────────────────────────
        // GETTERS públicos de solo lectura
        // ─────────────────────────────────────────────────────────────

        #[ink::test]
        fn getters_publicos_devuelven_datos_correctos() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar y ordenar dos compras
            c.publicar_producto("P".into(), "D".into(), 100, 5, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let id1 = c.crear_orden(0, 2).unwrap();
            let id2 = c.crear_orden(0, 1).unwrap();
            assert_ne!(id1, id2);

            // obtener_orden_pub
            let o1 = c.obtener_orden_pub(id1).expect("orden id1 debe existir");
            assert_eq!(o1.id, id1);
            assert!(c.obtener_orden_pub(99999).is_none());

            // listar_ordenes_por_usuario (bob participa en 2)
            let lista_bob = c.listar_ordenes_por_usuario(acc.bob);
            assert_eq!(lista_bob.len(), 2);

            // obtener_reputaciones (inicialmente 0)
            let rep_alice = c.obtener_reputaciones(acc.alice).unwrap();
            assert_eq!(rep_alice, (0, 0));

            // obtener_producto_pub
            let p0 = c.obtener_producto_pub(0).expect("producto 0 debe existir");
            assert_eq!(p0.nombre, "P");
            assert!(c.obtener_producto_pub(777).is_none());
        }


        // ------- Disputas -------
        #[ink::test]
        fn abrir_y_resolver_disputa_ok() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar producto y crear orden
            c.publicar_producto("P".into(), "D".into(), 100, 2, "Cat".into()).unwrap(); 
            // Sets automatizados para terminar más rápido :)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let orden_id = c.crear_orden(0, 1).unwrap();

            // Vendedor la marca enviada
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(orden_id).unwrap();

            // Comprador abre disputa
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            assert!(c.abrir_disputa(orden_id).is_ok());
            assert!(c.ordenes[orden_id as usize].disputa_abierta);

            // Vendedor la resuelve a favor del comprador
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            assert!(c.resolver_disputa(orden_id, true).is_ok());
            assert_eq!(c.ordenes[orden_id as usize].estado, EstadoOrden::Cancelada);
        }

        #[ink::test]
        fn abrir_disputa_falla_si_no_es_comprador() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar producto y crear orden
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let orden_id = c.crear_orden(0, 1).unwrap();

            // La orden aún está Pendiente (debe estar Enviada para abrir disputa)
            let r = c.abrir_disputa(orden_id);
            assert!(matches!(r, Err(SistemaError::EstadoInvalido)));

            // Vendedor la marca como Enviada
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(orden_id).unwrap();

            // Un tercero (no comprador) intenta abrir disputa
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let r = c.abrir_disputa(orden_id);
            assert!(matches!(r, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn abrir_disputa_falla_si_ya_existe() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar, comprar, enviar
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let orden_id = c.crear_orden(0, 1).unwrap();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(orden_id).unwrap();

            // Abrir disputa (OK)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            assert!(c.abrir_disputa(orden_id).is_ok());

            // Intentar abrir nuevamente
            let r2 = c.abrir_disputa(orden_id);
            assert!(matches!(r2, Err(SistemaError::DisputaAbierta)));
        }

        #[ink::test]
        fn resolver_disputa_falla_si_no_hay_disputa() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar y crear orden
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let orden_id = c.crear_orden(0, 1).unwrap();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(orden_id).unwrap();

            // Intentar resolver sin disputa abierta
            let r = c.resolver_disputa(orden_id, true);
            assert!(matches!(r, Err(SistemaError::NoHayDisputaAbierta)));
        }

        #[ink::test]
        fn resolver_disputa_falla_si_no_es_vendedor() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar, comprar y enviar
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let orden_id = c.crear_orden(0, 1).unwrap();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(orden_id).unwrap();

            // Comprador abre disputa
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            assert!(c.abrir_disputa(orden_id).is_ok());

            // Otro usuario intenta resolver
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.charlie);
            c.registrar_usuario(RolUsuario::Vendedor).unwrap();
            let r = c.resolver_disputa(orden_id, true);
            assert!(matches!(r, Err(SistemaError::NoEsRolCorrecto)));
        }

        #[ink::test]
        fn resolver_disputa_a_favor_del_vendedor_ok() {
            let mut c = setup_contract_con_vendedor();
            let acc = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Publicar, comprar, enviar y abrir disputa
            c.publicar_producto("P".into(), "D".into(), 100, 1, "Cat".into()).unwrap();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            c.registrar_usuario(RolUsuario::Comprador).unwrap();
            let orden_id = c.crear_orden(0, 1).unwrap();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            c.marcar_orden_como_enviada(orden_id).unwrap();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.bob);
            assert!(c.abrir_disputa(orden_id).is_ok());

            // Resolver en favor del vendedor
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(acc.alice);
            let r = c.resolver_disputa(orden_id, false);
            assert!(r.is_ok());
            assert_eq!(c.ordenes[orden_id as usize].estado, EstadoOrden::Recibida);
            assert!(!c.ordenes[orden_id as usize].disputa_abierta);
        }


    } // <-- cierre del mod tests
} // <-- cierre del mod marketplace_principal
pub use self::marketplace_principal::{MarketplacePrincipalRef, Orden, Producto, EstadoOrden};
