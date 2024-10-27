- [ ] agregarle algoritmos de
planificación de la CPU
    - [x] FCFS
    - [ ] SRT
    - [x] SJF
    - [ ] RR
    - [ ] HRRN
- [x] Agregar la funcionalidad para conocer el tamaño de ráfaga y tiempo de llegada
- [ ] La ráfaga será la cantidad de líneas a procesar de cada archivo
- [x] Cada línea del archivo representa una posición en memoria
- [x] Una vez cargado los archivos, debe existir una opción para agregar los tiempos de llegada para
cada proceso asignado en el CPU (manual y automático (random de 1 a 5)), esto será similar a
los ejercicios realizados.
- [ ] El tiempo inicial del primer archivo por defecto será de 1
- [ ] Debe contar con una ventana de configuración para escoger el algoritmo de planificación y
sus parámetros respectivos.
- [x] Debe contar con una ventana de configuración para el tamaño de la memoria primaria y
secundaria, además, la estrategia de asignación y sus respectivos parámetros.
- [ ] Cada CPU debe de tener la capacidad de ejecutar hasta 5 procesos. Usando la lógica de
asignación secuencial para cada uno, si existen procesos que no pudieron ubicarse deben
esperar por su asignación.
- [x] Si existe más de un CPU, la asignación entre CPU será aleatoria
- [ ] Si un proceso finaliza, este debe actualizar su estado y liberar su espacio en memoria
- [x] La ejecución será manual y automática
- [x] Cada instrucción de los archivos tendrá una duración de 1 segundo para así visualizar su
ejecución tal como lo practicado en clase
- [ ] Al final de cada ejecución debe manejar las estadísticas de su ejecución
    - [ ] Tiempo inicio y final
    - [ ] Tiempo de estancia (Turnaround) (Tiempo final – tiempo de llegada)
    - [ ] Tr / Ts
