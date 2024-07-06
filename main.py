import os
import pymongo
import dotenv
from flask import Flask
import uuid
from telegram.ext import Application, CommandHandler, ContextTypes, MessageHandler, filters
from telegram import ForceReply, Update

dotenv.load_dotenv()

db = pymongo.MongoClient(os.getenv("MONGOURL"))["vicweb"]
postsCollection = db["posts"]
print("Connected to database!", postsCollection)


async def mainTG() -> None:
    print(os.getenv("TG_TOKEN"))
    tgbot = Application.builder().token(os.getenv("TG_TOKEN")).build()
    tgbot.add_handler(MessageHandler(filters.ALL, handleMessage))
    tgbot.run_polling(allowed_updates=Update.MESSAGE)
    print("Telegram bot started!")

def handleMessage(update: Update, context: ContextTypes) -> None:
    print(f"New message recived! {update.message.chat.id}" )
    if (update.message.chat.id != int(os.getenv("TG_CHAT_ID"))):
        print(os.getenv("TG_CHAT_ID"))
        return
    print("New channel message recived!")
    Iid = str(uuid.uuid4())
    Mcontent = update.message.text

    dbdata = postsCollection.find_one({"content": Mcontent})

    if (dbdata == None):
        print (f"Message not found in database! Adding message to database with id: {Iid}")
        postsCollection.insert_one({"id": Iid, "content": Mcontent,"source": "telegram"})
        print("Post added to database!")
        return 
    print("Message skipped (already in database)!")

def webServer():
    app = Flask(__name__)

    @app.route('/')
    def hello_world():
        return 'Hello, Chat!'
    @app.route('/posts')
    def posts():
        print(postsCollection.find())
        return str(postsCollection.find())
    app.run()
    print(f"Webserver started at http://{app.host}:{app.port}")



if (__name__ == "__main__"):
    from tgbot import mainTG
    from webserver import webServer
    mainTG()
    webServer()