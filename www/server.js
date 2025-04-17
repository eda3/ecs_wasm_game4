const express = require('express');
const http = require('http');
const WebSocket = require('ws');

// HTTPサーバーの設定
const app = express();
const port = 8001; // HTTPサーバーのポート番号
const wsPort = 8101; // WebSocketサーバーのポート番号

// 静的ファイルのルートディレクトリを設定
app.use(express.static(__dirname));

// サーバーを作成
const server = http.createServer(app);

// WebSocketサーバーの設定
const wss = new WebSocket.Server({ port: wsPort }); // 専用ポートを使用

// WebSocket接続時の処理
wss.on('connection', function connection(ws) {
    console.log('クライアントが接続しました');

    // メッセージ受信時の処理
    ws.on('message', function incoming(message) {
        console.log('受信メッセージ: %s', message);

        // 全クライアントにメッセージをブロードキャスト
        wss.clients.forEach(function each(client) {
            if (client.readyState === WebSocket.OPEN) {
                client.send(message);
            }
        });
    });
});

// HTTPサーバーを起動
server.listen(port, () => {
    console.log(`サーバーが起動しました: http://162.43.8.148:${port}`);
    console.log(`WebSocketサーバー: ws://162.43.8.148:${wsPort}`);
}); 