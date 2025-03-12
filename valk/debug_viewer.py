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
        .event-type {
            background: #e9ecef;
            padding: 2px 6px;
            border-radius: 4px;
            font-size: 0.8em;
            min-width: 80px;
            text-align: center;
        }
        .request {
            color: #0c5460;
            background-color: #d1ecf1;
        }
        .response {
            color: #155724;
            background-color: #d4edda;
        }
        .screen-update {
            color: #856404;
            background-color: #fff3cd;
        }
        .cursor-update {
            color: #6f42c1;
            background-color: #e2d9f3;
        }
        #live-view {
            display: flex;
            margin: 20px 0;
            background: white;
            padding: 15px;
            border-radius: 4px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        #screen-container {
            position: relative;
            margin-right: 20px;
            flex: 1;
        }
        #live-screen {
            max-width: 100%;
            border: 1px solid #ddd;
            border-radius: 4px;
        }
        #cursor-position {
            position: absolute;
            right: 10px;
            top: 10px;
            background: rgba(0,0,0,0.7);
            color: white;
            padding: 5px 10px;
            border-radius: 4px;
            font-size: 0.9em;
        }
        #cursor-indicator {
            position: absolute;
            width: 16px;
            height: 16px;
            background: red;
            border-radius: 50%;
            transform: translate(-50%, -50%);
            pointer-events: none;
        }
        #sidebar {
            width: 250px;
            padding: 10px;
            background: #f8f9fa;
            border-radius: 4px;
        }
        .info-item {
            margin-bottom: 10px;
        }
        .info-label {
            font-weight: bold;
            margin-bottom: 5px;
        }
        .info-value {
            color: #555;
            word-break: break-all;
        }
        .tabs {
            display: flex;
            margin-bottom: 10px;
        }
        .tab {
            padding: 8px 16px;
            background: #e9ecef;
            border-radius: 4px 4px 0 0;
            cursor: pointer;
            margin-right: 5px;
        }
        .tab.active {
            background: white;
            border-bottom: 2px solid #007bff;
        }
    </style>
</head>
<body>
    <h1>Valk Debug Viewer</h1>
    <div id="status" class="disconnected">Disconnected</div>
    
    <div class="tabs">
        <div class="tab active" onclick="switchTab('live-view')">Live View</div>
        <div class="tab" onclick="switchTab('events-log')">Event Log</div>
    </div>
    
    <div id="live-view">
        <div id="screen-container">
            <img id="live-screen" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=" alt="Screen">
            <div id="cursor-position">X: 0, Y: 0</div>
            <div id="cursor-indicator"></div>
        </div>
        <div id="sidebar">
            <div class="info-item">
                <div class="info-label">Last Update:</div>
                <div id="last-update" class="info-value">-</div>
            </div>
            <div class="info-item">
                <div class="info-label">Resolution:</div>
                <div id="resolution" class="info-value">-</div>
            </div>
            <div class="info-item">
                <div class="info-label">Last Action:</div>
                <div id="last-action" class="info-value">-</div>
            </div>
        </div>
    </div>
    
    <div id="events-log" style="display: none;">
        <button class="clear-btn" onclick="clearEvents()">Clear Events</button>
        <div id="events"></div>
    </div>
    
    <div id="screenshot-modal" class="screenshot-modal" onclick="hideScreenshot()">
        <img id="modal-image">
    </div>

    <script>
        let ws = null;
        const eventsDiv = document.getElementById('events');
        const statusDiv = document.getElementById('status');
        const modal = document.getElementById('screenshot-modal');
        const modalImg = document.getElementById('modal-image');
        const liveScreen = document.getElementById('live-screen');
        const cursorPosition = document.getElementById('cursor-position');
        const cursorIndicator = document.getElementById('cursor-indicator');
        const lastUpdateElem = document.getElementById('last-update');
        const resolutionElem = document.getElementById('resolution');
        const lastActionElem = document.getElementById('last-action');
        
        let currentCursorX = 0;
        let currentCursorY = 0;
        let screenWidth = 0;
        let screenHeight = 0;

        function switchTab(tabId) {
            // Hide all tabs
            document.getElementById('live-view').style.display = 'none';
            document.getElementById('events-log').style.display = 'none';
            
            // Show selected tab
            document.getElementById(tabId).style.display = tabId === 'live-view' ? 'flex' : 'block';
            
            // Update tab styling
            document.querySelectorAll('.tab').forEach(tab => {
                tab.classList.remove('active');
            });
            
            // Find and activate the clicked tab
            document.querySelectorAll('.tab').forEach(tab => {
                if (tab.textContent.toLowerCase().includes(tabId.replace('-', ' '))) {
                    tab.classList.add('active');
                }
            });
        }

        function connect() {
            ws = new WebSocket('ws://VALK_BASE_URL/v1/monitor');
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
                processEvent(data);
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
        
        function updateCursorPosition(x, y) {
            currentCursorX = x;
            currentCursorY = y;
            cursorPosition.textContent = `X: ${x}, Y: ${y}`;
            
            // Update the visual cursor indicator position
            // Calculate percentage position relative to screen dimensions
            if (screenWidth > 0 && screenHeight > 0) {
                const containerRect = document.getElementById('screen-container').getBoundingClientRect();
                const screenRect = liveScreen.getBoundingClientRect();
                
                // Calculate the position within the image
                const xPercent = x / screenWidth;
                const yPercent = y / screenHeight;
                
                // Position the cursor indicator
                cursorIndicator.style.left = `${screenRect.left - containerRect.left + (xPercent * screenRect.width)}px`;
                cursorIndicator.style.top = `${screenRect.top - containerRect.top + (yPercent * screenRect.height)}px`;
                cursorIndicator.style.display = 'block';
            }
        }

        function processEvent(data) {
            // Check event type
            if (data.event_type === 'screen_update') {
                // Update the live screen
                liveScreen.src = 'data:image/png;base64,' + data.data.image;
                
                // Update screen dimensions
                screenWidth = data.data.width;
                screenHeight = data.data.height;
                
                // Update info sidebar
                const timestamp = new Date(data.data.timestamp).toLocaleTimeString();
                lastUpdateElem.textContent = timestamp;
                resolutionElem.textContent = `${screenWidth}x${screenHeight}`;
                
                // Update cursor position on the new screen
                updateCursorPosition(currentCursorX, currentCursorY);
                
                // Add to event log
                addScreenUpdateEvent(data);
            } 
            else if (data.event_type === 'cursor_update') {
                // Update cursor position
                updateCursorPosition(data.data.x, data.data.y);
                
                // Add to event log
                addCursorUpdateEvent(data);
            }
            else if (data.event_type === 'action_request') {
                // Update last action in sidebar
                lastActionElem.textContent = `Request: ${data.data.action.type}`;
                
                // Add to event log
                addActionRequestEvent(data.data);
            }
            else if (data.event_type === 'action_response') {
                // Update last action in sidebar
                lastActionElem.textContent = `Response: ${data.data.action.type} (${data.data.status})`;
                
                // Add to event log
                addActionResponseEvent(data.data);
            }
        }

        function addScreenUpdateEvent(data) {
            const eventDiv = document.createElement('div');
            eventDiv.className = 'event';
            
            const time = new Date(data.data.timestamp).toLocaleTimeString();
            
            eventDiv.innerHTML = `
                <div class="event-type screen-update">Screen</div>
                <div class="event-time">${time}</div>
                <div class="event-action">${data.data.width}x${data.data.height}</div>
                <div class="event-data">
                    <img src="data:image/png;base64,${data.data.image}" 
                         class="screenshot-preview"
                         onclick="showScreenshot('${data.data.image}')">
                </div>
            `;
            
            eventsDiv.insertBefore(eventDiv, eventsDiv.firstChild);
        }

        function addCursorUpdateEvent(data) {
            const eventDiv = document.createElement('div');
            eventDiv.className = 'event';
            
            const time = new Date(data.data.timestamp).toLocaleTimeString();
            
            eventDiv.innerHTML = `
                <div class="event-type cursor-update">Cursor</div>
                <div class="event-time">${time}</div>
                <div class="event-data">Position: x=${data.data.x}, y=${data.data.y}</div>
            `;
            
            eventsDiv.insertBefore(eventDiv, eventsDiv.firstChild);
        }

        function addActionRequestEvent(request) {
            const eventDiv = document.createElement('div');
            eventDiv.className = 'event';
            
            const time = new Date(request.timestamp || Date.now()).toLocaleTimeString();
            const actionType = request.action.type || "unknown";
            
            eventDiv.innerHTML = `
                <div class="event-type request">Request</div>
                <div class="event-time">${time}</div>
                <div class="event-action">${actionType}</div>
                <div class="event-data">ID: ${request.id}</div>
            `;
            
            eventsDiv.insertBefore(eventDiv, eventsDiv.firstChild);
        }

        function addActionResponseEvent(response) {
            const eventDiv = document.createElement('div');
            eventDiv.className = `event ${response.status === 'success' ? 'success' : 'error'}`;
            
            const time = new Date(response.timestamp).toLocaleTimeString();
            const actionType = response.action.type || "unknown";
            const status = response.status || "unknown";
            
            let dataContent = '';
            // Special handling for screenshot data
            if (actionType === 'screenshot' && response.data && response.data.image) {
                dataContent = `
                    <img src="data:image/png;base64,${response.data.image}" 
                         class="screenshot-preview"
                         onclick="showScreenshot('${response.data.image}')">
                `;
            } 
            // Special handling for cursor position data
            else if (actionType === 'cursor_position' && response.data) {
                dataContent = `Position: x=${response.data.x}, y=${response.data.y}`;
            }
            // Error handling
            else if (response.error) {
                dataContent = `Error: ${formatData(response.error)}`;
            }
            // Generic data output
            else if (response.data) {
                dataContent = formatData(response.data);
            }
            
            eventDiv.innerHTML = `
                <div class="event-type response">Response</div>
                <div class="event-time">${time}</div>
                <div class="event-action">${actionType}</div>
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
