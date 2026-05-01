#include <iostream>
#include <string>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#define SOCKET_PATH "/tmp/sml_ollama_bridge.sock"

// Este es el único código C++ que se inyectará en llama.cpp
// Su única función es enviar el comando SML al socket seguro de Rust y devolver la respuesta.
// Esto garantiza que cualquier fallo ocurra en el lado seguro de Rust y no cuelgue el motor de C++.

std::string execute_sml_via_ipc(const std::string& sml_command) {
    int sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock < 0) {
        return "[ERR:IPC_SOCKET_FAILED]";
    }

    struct sockaddr_un server_addr;
    server_addr.sun_family = AF_UNIX;
    strncpy(server_addr.sun_path, SOCKET_PATH, sizeof(server_addr.sun_path) - 1);

    if (connect(sock, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        close(sock);
        return "[ERR:IPC_CONNECT_FAILED] Rust bridge might be down.";
    }

    // Enviar el comando
    if (send(sock, sml_command.c_str(), sml_command.length(), 0) < 0) {
        close(sock);
        return "[ERR:IPC_SEND_FAILED]";
    }

    // Leer la respuesta
    char buffer[4096] = {0};
    int bytes_read = read(sock, buffer, sizeof(buffer) - 1);
    close(sock);

    if (bytes_read < 0) {
        return "[ERR:IPC_READ_FAILED]";
    }

    return std::string(buffer, bytes_read);
}

// Ejemplo de uso (Hook simulado)
int main() {
    std::cout << "Simulando Ollama/Llama.cpp generando un comando SML..." << std::endl;
    std::string test_cmd = "@[read:Cargo.toml]";
    
    std::string result = execute_sml_via_ipc(test_cmd);
    
    std::cout << "Respuesta recibida del puente Rust (IPC):\\n" << result << std::endl;
    return 0;
}
