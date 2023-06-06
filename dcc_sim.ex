defmodule DccSimulation do
    require Logger
    use GenServer

    @ip {127, 0, 0, 1}
    @port 6000

    def send_message(message) do
      GenServer.cast(__MODULE__, {:message, message})
    end

    def heart_beat() do
    list =   [
      254, 254, 0, 223, 1, 2, 1, 0, 0, 0, 0, 0, 0, 9, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 139,
      109, 49, 1, 0, 0, 0, 1, 1, 0, 0, 1, 2, 0, 0, 1, 3, 0, 0, 1, 4, 0, 0, 1, 5, 0, 0, 1, 6, 0,
      0, 1, 7, 0, 0, 1, 8, 0, 0, 1, 9, 0, 0, 1, 10, 0, 0, 1, 11, 0, 0, 1, 12, 0, 0, 1, 13, 0, 0,
      1, 14, 0, 0, 1, 15, 0, 0, 1, 16, 0, 0, 1, 17, 0, 0, 1, 18, 0, 0, 1, 19, 0, 0, 2, 0, 0, 0,
      2, 1, 0, 0, 2, 2, 0, 0, 2, 3, 0, 0, 2, 4, 0, 0, 3, 0, 0, 0, 3, 1, 0, 0, 3, 2, 0, 0, 3, 3,
      0, 0, 3, 4, 0, 0, 3, 5, 0, 0, 3, 6, 0, 0, 4, 0, 15, 160, 4, 1, 15, 160, 4, 2, 15, 160, 4,
      3, 15, 160, 4, 4, 15, 160, 7, 0, 0, 44, 7, 1, 0, 44, 7, 2, 0, 40, 7, 3, 11, 180, 7, 4, 11,
      179, 7, 5, 11, 180, 7, 6, 11, 179, 8, 0, 7, 189, 8, 1, 7, 189, 8, 2, 7, 189, 253, 0, 1, 13,
      254, 0, 1, 1, 0x14, 0x2e,
  ]
     bin = for x <-list, into: <<>> do
      <<x>>
     end

      send_message(bin)
    end

    def start do
      GenServer.start(__MODULE__, %{socket: nil},name: __MODULE__)
    end

    def init(state) do
      send(self(), :connect)
      {:ok, state}
    end

    def handle_info(:connect, state) do
      Logger.info "Connecting to #{:inet.ntoa(@ip)}:#{@port}"

      case :gen_tcp.connect(@ip, @port, [:binary, active: true]) do
        {:ok, socket} ->
          {:noreply, %{state | socket: socket}}
        {:error, reason} ->
          disconnect(state, reason)
      end
    end

    def handle_info({:tcp, _, data}, state) do
      Logger.info "Received #{inspect data}"

      {:noreply, state}
    end

    def handle_info({:tcp_closed, _}, state), do: {:stop, :normal, state}
    def handle_info({:tcp_error, _}, state), do: {:stop, :normal, state}

    def handle_cast({:message, message}, %{socket: socket} = state) do
      Logger.info "Sending #{message}"

      :ok = :gen_tcp.send(socket, message)
      {:noreply, state}
    end

    def disconnect(state, reason) do
      Logger.info "Disconnected: #{reason}"
      {:stop, :normal, state}
    end
  end
