from telegram import ForceReply, Update
from telegram.ext import Application, CommandHandler, ContextTypes, MessageHandler, filters
import os
import uuid
import pymongo
import dotenv

dotenv.load_dotenv()

db = pymongo.MongoClient(os.getenv("MONGOURL"))["vicweb"]
postsCollection = db["posts"]
print("Connected to database!", postsCollection)
def main() -> None:
    tgbot = Application.builder().token(os.getenv("TG_TOKEN")).build()
    tgbot.add_handler(MessageHandler(filters.ALL, handleMessage))
    tgbot.run_polling(allowed_updates=Update.MESSAGE)
    print("Telegram bot started!")

async def handleMessage(update: Update, context: ContextTypes) -> None:
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

    

print(__name__)

if (__name__ == "__main__"):
    main()

