VIEWER_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Valk Debug Viewer</title>
    <style>
        body {
            font-family: system-ui, sans-serif;
            margin: 20px;
            background: #f0f0f0;
        }
        #status {
            padding: 10px;
            margin-bottom: 10px;
            border-radius: 4px;
        }
        .connected {
            background: #d4edda;
            color: #155724;
        }
        .disconnected {
            background: #f8d7da;
            color: #721c24;
        }
        #events {
            height: 500px;
            overflow-y: auto;
            background: white;
            padding: 10px;
            border-radius: 4px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .event {
            padding: 8px;
            margin: 4px 0;
            border-left: 4px solid #ddd;
            background: #f8f9fa;
            display: flex;
            gap: 16px;
            align-items: flex-start;
        }
        .event.success {
            border-left-color: #28a745;
        }
        .event.error {
            border-left-color: #dc3545;
        }
        .event-time {
            color: #666;
            font-size: 0.9em;
            white-space: nowrap;
            min-width: 80px;
        }
        .event-action {
            font-weight: 500;
            min-width: 100px;
        }
        .event-status {
            min-width: 60px;
        }
        .event-data {
            flex: 1;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
        .clear-btn {
            margin: 10px 0;
            padding: 8px 16px;
            background: #6c757d;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        .clear-btn:hover {
            background: #5a6268;
        }
        .screenshot-preview {
            max-width: 200px;
            max-height: 150px;
            margin-top: 8px;
            border-radius: 4px;
            cursor: pointer;
        }
        .screenshot-modal {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0,0,0,0.8);
            z-index: 1000;
            padding: 20px;
            cursor: pointer;
        }
        .screenshot-modal img {
            max-width: 90%;
            max-height: 90%;
            margin: auto;
            display: block;
        }
    </style>
</head>
<body>
    <h1>Valk Debug Viewer</h1>
    <div id="status" class="disconnected">Disconnected</div>
    <button class="clear-btn" onclick="clearEvents()">Clear Events</button>
    <div id="events"></div>
    <div id="screenshot-modal" class="screenshot-modal" onclick="hideScreenshot()">

        <img id="modal-image">
    </div>

    <script>
        let ws = null;
        const eventsDiv = document.getElementById('events');
        const statusDiv = document.getElementById('status');
        const modal = document.getElementById('screenshot-modal');
        const modalImg = document.getElementById('modal-image');

        function connect() {
            ws = new WebSocket('ws://VALK_BASE_URL/ws/monitor');
            ws.onopen = () => {
                statusDiv.textContent = 'Connected';
                statusDiv.className = 'connected';
            };
            ws.onclose = () => {
                statusDiv.textContent = 'Disconnected. Reconnecting in 3s...';
                statusDiv.className = 'disconnected';
                setTimeout(connect, 3000);
            };
            ws.onerror = () => {
                statusDiv.textContent = 'Connection Error';
                statusDiv.className = 'disconnected';
            };
            ws.onmessage = (event) => {
                const data = JSON.parse(event.data);
                addEvent(data);
            };
        }

        function formatData(data) {
            if (!data) return '';
            const str = JSON.stringify(data);
            return str.length > 100 ? str.substring(0, 100) + '...' : str;
        }

        function showScreenshot(imageData) {
            modalImg.src = 'data:image/png;base64,' + imageData;
            modal.style.display = 'block';
        }

        function hideScreenshot() {
            modal.style.display = 'none';
        }

        function addEvent(data) {
            const eventDiv = document.createElement('div');
            eventDiv.className = 'event ' + (data.result.Ok ? 'success' : 'error');
            
            const time = new Date(data.timestamp).toLocaleTimeString();
            const action = typeof data.action === 'string' ? data.action : JSON.stringify(data.action);
            const status = data.result.Ok ? 'Ok' : 'Error';
            const resultData = data.result.Ok || data.result.Err;
            
            let dataContent = '';
            if (action === 'Screenshot' && resultData.data?.image) {
                dataContent = `
                    <img src="data:image/png;base64,${resultData.data.image}" 
                         class="screenshot-preview"
                         onclick="showScreenshot('${resultData.data.image}')">
                `;
            } else {
                dataContent = formatData(resultData);
            }

            eventDiv.innerHTML = `
                <div class="event-time">${time}</div>
                <div class="event-action">${action}</div>
                <div class="event-status">${status}</div>
                <div class="event-data">${dataContent}</div>
            `;
            
            eventsDiv.insertBefore(eventDiv, eventsDiv.firstChild);
        }

        function clearEvents() {
            eventsDiv.innerHTML = '';
        }

        // Start connection
        connect();
    </script>
</body>
</html>
"""
