class EventManager {
    private events: { [eventName: string]: Function[] } = {};

    on(eventName: string, callback: Function) {
        if (!this.events[eventName]) {
            this.events[eventName] = [];
        }
        this.events[eventName].push(callback);
    }

    off(eventName: string, callback: Function) {
        if (this.events[eventName]) {
            const index = this.events[eventName].indexOf(callback);
            if (index !== -1) {
                this.events[eventName].splice(index, 1);
            }
        }
    }

    emit(eventName: string, ...args: any[]) {
        if (this.events[eventName]) {
            this.events[eventName].forEach((callback) => {
                callback(...args);
            });
        }
    }
}

/**
 * This class will contain the connection with the websocket
 */
class WebsocketNetwork {
    private url: string = "ws://127.0.0.1:9090/root/private/socket";
    private token: string | null = null;
    private connection: WebSocket | null;
    public events: EventManager;

    constructor() {
        this.connection = null;
        this.events = new EventManager();
    }

    /**
     * Connect to the websocket
     *
     * @param id
     * @param passwd
     * @param host
     */
    connect(id: string, passwd: string, host: string) {
        try {
            this.connection = new WebSocket(this.url);
            this.connection.addEventListener("open", () => this.onopen(id, passwd))
            this.connection.addEventListener('error', (e) => this.onerror(e, this.events));
            this.connection.addEventListener("message", this.onmessage);
        } catch(err: any) {
            console.log("error received: ", err.toString())
            this.events.emit("connection_refused");
        }
    }

    onopen(id: string, passwd: string) {
        if (this.connection) {
            this.connection.send(JSON.stringify({
                op: 2,
                payload: { id, passwd }
            }))
        } else throw new Error("Websocket is null")
    }

    /**
     * Called for each message received from the websocket
     *
     * @param message
     */
    onmessage(message: MessageEvent) {
        console.log("message received:", message);

        if (!("op" in message) || !("payload" in message)) throw new Error(`Invalid websocket message received: ${message}`);

        let msg = message.data as WebsocketMessage;

        switch(message.op) {
            case OpCode.Connected: {
                let connect_box = document.querySelector(".connection");
                if (connect_box) connect_box.classList.add("connected")
                break;
            }
        }
    }

    /**
     * Called when an error occurred inside the websocket
     *
     * @param error
     */
    onerror(error: Event, event_manager: EventManager){
        console.log(this.events, error);
        console.error('WebSocket error:', error);
        this.events.emit('connection_refused');
    }
}

enum OpCode {
    Connected = 0,
    Heartbeat = 1,
    Credentials = 2,
    TransmitToken = 3
}

type WebsocketMessage = {
    op: OpCode,
    payload: any
};



export { WebsocketNetwork, EventManager };
export type { WebsocketMessage, OpCode };