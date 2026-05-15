#[derive(Debug)]
struct Vuelo {
    id: String,
    altitud: u32, // Clave del árbol AVL
}

struct Nodo {
    vuelo: Vuelo,
    izquierdo: Option<Box<Nodo>>,
    derecho: Option<Box<Nodo>>,
    altura: i32,
}

impl Nodo {
    fn nuevo(vuelo: Vuelo) -> Self {
        Nodo {
            vuelo,
            izquierdo: None,
            derecho: None,
            altura: 1,
        }
    }
}

// ==========================================================
// UTILIDADES DE BALANCEO
// ==========================================================

fn obtener_altura(nodo: &Option<Box<Nodo>>) -> i32 {
    nodo.as_ref().map_or(0, |n| n.altura)
}

fn actualizar_altura(nodo: &mut Nodo) {
    nodo.altura = 1 + std::cmp::max(
        obtener_altura(&nodo.izquierdo),
        obtener_altura(&nodo.derecho),
    );
}

fn obtener_balance(nodo: &Nodo) -> i32 {
    obtener_altura(&nodo.izquierdo) - obtener_altura(&nodo.derecho)
}

/*
    EXPLICACIÓN DE Option::take()

    En Rust, no podemos mover directamente un hijo de un nodo si solo tenemos
    una referencia mutable al nodo, porque Rust protege la propiedad de los datos.

    Por eso usamos .take().

    Ejemplo:
    y.izquierdo.take()

    Eso hace dos cosas:
    1. Extrae el valor que estaba dentro de Option<Box<Nodo>>.
    2. Deja None en su lugar.

    De esta forma, Rust permite mover ese hijo de forma segura sin dejar
    el nodo original en un estado inválido.

    Esto es muy útil en rotaciones AVL porque necesitamos reorganizar
    los punteros izquierdo y derecho.
*/

fn rotar_derecha(mut y: Box<Nodo>) -> Box<Nodo> {
    let mut x = y.izquierdo.take().expect("Error de radar: no hay hijo izquierdo");

    y.izquierdo = x.derecho.take();

    actualizar_altura(&mut y);

    x.derecho = Some(y);

    actualizar_altura(&mut x);

    x
}

fn rotar_izquierda(mut x: Box<Nodo>) -> Box<Nodo> {
    let mut y = x.derecho.take().expect("Error de radar: no hay hijo derecho");

    x.derecho = y.izquierdo.take();

    actualizar_altura(&mut x);

    y.izquierdo = Some(x);

    actualizar_altura(&mut y);

    y
}

fn balancear(mut nodo: Box<Nodo>) -> Box<Nodo> {
    actualizar_altura(&mut nodo);

    let balance = obtener_balance(&nodo);

    // Caso izquierda pesada
    if balance > 1 {
        if obtener_balance(nodo.izquierdo.as_ref().unwrap()) < 0 {
            let hijo_izq = nodo.izquierdo.take().unwrap();
            nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        }

        return rotar_derecha(nodo);
    }

    // Caso derecha pesada
    if balance < -1 {
        if obtener_balance(nodo.derecho.as_ref().unwrap()) > 0 {
            let hijo_der = nodo.derecho.take().unwrap();
            nodo.derecho = Some(rotar_derecha(hijo_der));
        }

        return rotar_izquierda(nodo);
    }

    nodo
}

// ==========================================================
// INSERCIÓN
// ==========================================================

fn insertar(nodo_opt: Option<Box<Nodo>>, vuelo: Vuelo) -> Box<Nodo> {
    let altitud_nueva = vuelo.altitud;

    let mut nodo = match nodo_opt {
        None => return Box::new(Nodo::nuevo(vuelo)),
        Some(n) => n,
    };

    if altitud_nueva < nodo.vuelo.altitud {
        nodo.izquierdo = Some(insertar(nodo.izquierdo.take(), vuelo));
    } else if altitud_nueva > nodo.vuelo.altitud {
        nodo.derecho = Some(insertar(nodo.derecho.take(), vuelo));
    } else {
        println!("Ya existe un vuelo con altitud {}", altitud_nueva);
        return nodo;
    }

    actualizar_altura(&mut nodo);

    let balance = obtener_balance(&nodo);

    // Caso Izquierda-Izquierda
    if balance > 1 && altitud_nueva < nodo.izquierdo.as_ref().unwrap().vuelo.altitud {
        return rotar_derecha(nodo);
    }

    // Caso Derecha-Derecha
    if balance < -1 && altitud_nueva > nodo.derecho.as_ref().unwrap().vuelo.altitud {
        return rotar_izquierda(nodo);
    }

    // Caso Izquierda-Derecha
    if balance > 1 && altitud_nueva > nodo.izquierdo.as_ref().unwrap().vuelo.altitud {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return rotar_derecha(nodo);
    }

    // Caso Derecha-Izquierda
    if balance < -1 && altitud_nueva < nodo.derecho.as_ref().unwrap().vuelo.altitud {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return rotar_izquierda(nodo);
    }

    nodo
}

// ==========================================================
// FASE 2: BÚSQUEDA DE VUELO
// ==========================================================

fn buscar_vuelo(nodo: &Option<Box<Nodo>>, altitud: u32) -> Option<&Vuelo> {
    match nodo {
        None => None,

        Some(n) => {
            if altitud == n.vuelo.altitud {
                Some(&n.vuelo)
            } else if altitud < n.vuelo.altitud {
                buscar_vuelo(&n.izquierdo, altitud)
            } else {
                buscar_vuelo(&n.derecho, altitud)
            }
        }
    }
}

// ==========================================================
// FASE 3: ELIMINACIÓN CON PREDECESOR IN-ORDER
// ==========================================================

fn extraer_max(mut nodo: Box<Nodo>) -> (Option<Box<Nodo>>, Vuelo) {
    if nodo.derecho.is_none() {
        let izquierdo = nodo.izquierdo.take();
        let vuelo_max = nodo.vuelo;

        return (izquierdo, vuelo_max);
    }

    let derecho = nodo.derecho.take().unwrap();

    let (nuevo_derecho, vuelo_max) = extraer_max(derecho);

    nodo.derecho = nuevo_derecho;

    let nodo_balanceado = balancear(nodo);

    (Some(nodo_balanceado), vuelo_max)
}

fn eliminar_vuelo(nodo_opt: Option<Box<Nodo>>, altitud: u32) -> Option<Box<Nodo>> {
    let mut nodo = match nodo_opt {
        None => return None,
        Some(n) => n,
    };

    if altitud < nodo.vuelo.altitud {
        nodo.izquierdo = eliminar_vuelo(nodo.izquierdo.take(), altitud);
    } else if altitud > nodo.vuelo.altitud {
        nodo.derecho = eliminar_vuelo(nodo.derecho.take(), altitud);
    } else {
        // Caso 1: nodo sin hijos
        if nodo.izquierdo.is_none() && nodo.derecho.is_none() {
            return None;
        }

        // Caso 2: nodo con solo hijo derecho
        if nodo.izquierdo.is_none() {
            return nodo.derecho;
        }

        // Caso 2: nodo con solo hijo izquierdo
        if nodo.derecho.is_none() {
            return nodo.izquierdo;
        }

        // Caso 3: nodo con dos hijos
        // Usamos el predecesor in-order:
        // el valor más alto del subárbol izquierdo.
        let izquierdo = nodo.izquierdo.take().unwrap();

        let (nuevo_izquierdo, vuelo_predecesor) = extraer_max(izquierdo);

        nodo.vuelo = vuelo_predecesor;
        nodo.izquierdo = nuevo_izquierdo;
    }

    Some(balancear(nodo))
}

// ==========================================================
// FASE 4: ALERTA DE PROXIMIDAD
// ==========================================================

fn vuelos_en_rango(nodo: &Option<Box<Nodo>>, min: u32, max: u32) -> usize {
    match nodo {
        None => 0,

        Some(n) => {
            let mut contador = 0;

            if n.vuelo.altitud >= min && n.vuelo.altitud <= max {
                contador += 1;
            }

            if min < n.vuelo.altitud {
                contador += vuelos_en_rango(&n.izquierdo, min, max);
            }

            if max > n.vuelo.altitud {
                contador += vuelos_en_rango(&n.derecho, min, max);
            }

            contador
        }
    }
}

// ==========================================================
// FUNCIONES PARA MOSTRAR EL ÁRBOL
// ==========================================================

fn imprimir_inorden(nodo: &Option<Box<Nodo>>) {
    if let Some(n) = nodo {
        imprimir_inorden(&n.izquierdo);

        println!(
            "Vuelo: {} | Altitud: {} pies | Altura del nodo: {}",
            n.vuelo.id, n.vuelo.altitud, n.altura
        );

        imprimir_inorden(&n.derecho);
    }
}

fn imprimir_arbol(nodo: &Option<Box<Nodo>>, espacio: usize) {
    if let Some(n) = nodo {
        imprimir_arbol(&n.derecho, espacio + 5);

        println!(
            "{:espacio$}{} ({})",
            "",
            n.vuelo.altitud,
            n.vuelo.id,
            espacio = espacio
        );

        imprimir_arbol(&n.izquierdo, espacio + 5);
    }
}

// ==========================================================
// MAIN
// ==========================================================

fn main() {
    let mut radar: Option<Box<Nodo>> = None;

    let datos = vec![
        ("AV123", 5000),
        ("UA456", 3000),
        ("IB101", 2000),
        ("AF999", 4000),
        ("TA222", 3500),
        ("AM777", 6000),
    ];

    for (id, alt) in datos {
        let vuelo = Vuelo {
            id: id.to_string(),
            altitud: alt,
        };

        radar = Some(insertar(radar.take(), vuelo));
    }

    println!("--- Radar de Control Aéreo AVL ---");
    println!("\nÁrbol mostrado de forma lateral:");
    imprimir_arbol(&radar, 0);

    println!("\nListado inorden, de menor a mayor altitud:");
    imprimir_inorden(&radar);

    println!("\n--- Búsqueda de vuelo ---");

    let altitud_buscada = 3500;

    match buscar_vuelo(&radar, altitud_buscada) {
        Some(vuelo) => {
            println!(
                "Vuelo encontrado: {} a {} pies",
                vuelo.id, vuelo.altitud
            );
        }

        None => {
            println!("No se encontró vuelo a {} pies", altitud_buscada);
        }
    }

    println!("\n--- Alerta de proximidad ---");

    let min = 3000;
    let max = 5000;

    let cantidad = vuelos_en_rango(&radar, min, max);

    println!(
        "Cantidad de vuelos entre {} y {} pies: {}",
        min, max, cantidad
    );

    println!("\n--- Eliminando vuelo a 3000 pies ---");

    radar = eliminar_vuelo(radar.take(), 3000);

    println!("\nÁrbol después de eliminar:");
    imprimir_arbol(&radar, 0);

    println!("\nListado inorden después de eliminar:");
    imprimir_inorden(&radar);
}
/*
    EXPLICACIÓN DE Box<Nodo>

    Usamos Box<Nodo> porque un árbol es una estructura recursiva.

    Si escribiéramos esto directamente:

    izquierdo: Option<Nodo>
    derecho: Option<Nodo>

    Rust no podría calcular el tamaño de Nodo en tiempo de compilación,
    porque un Nodo tendría dentro otro Nodo, y ese otro Nodo tendría otro,
    y así infinitamente.

    Con Box<Nodo>, el nodo no guarda otro Nodo directamente.
    Guarda un puntero hacia un Nodo almacenado en el heap.

    Eso hace que el tamaño de la estructura sea conocido en tiempo de compilación,
    porque Box tiene un tamaño fijo.
*/
/*
    PRUEBA DE ESCRITORIO

    Altitudes insertadas:
    [5000, 3000, 2000, 4000, 3500, 6000]

    1) Insertar 5000:
       5000

    2) Insertar 3000:
          5000
         /
       3000

    3) Insertar 2000:
            5000
           /
         3000
         /
       2000

       Se desbalancea en 5000.
       Caso Izquierda-Izquierda.
       Rotación simple a la derecha.

       Resultado:
            3000
           /    \
        2000    5000

    4) Insertar 4000:
            3000
           /    \
        2000    5000
                /
              4000

    5) Insertar 3500:
            3000
           /    \
        2000    5000
                /
              4000
              /
            3500

       Se desbalancea en 3000.
       Caso Derecha-Izquierda.
       Rotación doble:
       primero rotación derecha en 5000,
       luego rotación izquierda en 3000.

       Resultado:
              4000
             /    \
          3000    5000
          /  \
       2000 3500

    6) Insertar 6000:
              4000
             /    \
          3000    5000
          /  \       \
       2000 3500     6000

    Árbol final balanceado.
*/